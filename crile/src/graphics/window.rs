use std::sync::Arc;

use crate::{EventKind, WGPUContext, WindowId};
pub use winit::window::CursorIcon;

pub struct Window {
    pub(crate) winit: Arc<winit::window::Window>,
    pub viewport: WindowViewport,
}

impl Window {
    pub fn new(
        wgpu: &WGPUContext,
        event_loop: &winit::event_loop::EventLoopWindowTarget<()>,
    ) -> Self {
        let winit = Arc::new(
            winit::window::WindowBuilder::new()
                .with_title("Crile")
                .build(event_loop)
                .expect("failed to create window"),
        );

        Self {
            viewport: WindowViewport::new(wgpu, winit.clone()),
            winit,
        }
    }

    /// Returns the width and height of the window
    /// Guaranties to be at least 1
    pub fn size(&self) -> glam::UVec2 {
        let size = self.winit.inner_size();
        glam::UVec2::new(size.width.max(1), size.height.max(1))
    }

    pub fn scale_factor(&self) -> f64 {
        self.winit.scale_factor()
    }

    pub fn id(&self) -> WindowId {
        self.winit.id()
    }

    pub fn process_event(&mut self, event: &EventKind, wgpu: &WGPUContext) {
        if let EventKind::WindowResize { size } = event {
            self.viewport.resize(*size, wgpu)
        };
    }

    /// Sets the cursor icon
    /// Use None to make it invisible
    pub fn set_cursor_icon(&self, icon: Option<CursorIcon>) {
        if let Some(icon) = icon {
            self.winit.set_cursor_icon(icon);
        } else {
            self.winit.set_cursor_visible(false);
        }
    }

    /// Tries to lock/unlock the cursor within the window
    /// Returns whether or not it was successful
    pub fn set_cursor_lock(&self, lock: bool) -> bool {
        if lock {
            self.winit
                .set_cursor_grab(winit::window::CursorGrabMode::Locked)
                .or_else(|_| {
                    self.winit
                        .set_cursor_grab(winit::window::CursorGrabMode::Confined)
                })
                .is_ok()
        } else {
            self.winit
                .set_cursor_grab(winit::window::CursorGrabMode::None)
                .is_ok()
        }
    }
}

pub struct WindowViewport {
    pub(crate) surface: wgpu::Surface<'static>,
    pub(crate) surface_config: wgpu::SurfaceConfiguration,
}

impl WindowViewport {
    pub fn new(wgpu: &WGPUContext, winit: Arc<winit::window::Window>) -> Self {
        let size = winit.inner_size();
        let surface = wgpu
            .instance
            .create_surface(winit.clone())
            .expect("Failed to create surface!");

        let surface_config = surface
            .get_default_config(&wgpu.adapter, size.width, size.height)
            .unwrap();
        surface.configure(&wgpu.device, &surface_config);

        Self {
            surface,
            surface_config,
        }
    }

    pub fn resize(&mut self, size: glam::UVec2, wgpu: &WGPUContext) {
        self.surface_config.width = size.x;
        self.surface_config.height = size.y;
        self.surface.configure(&wgpu.device, &self.surface_config);
    }

    /// Tries to enable/disable vsync
    pub fn set_vsync(&mut self, wgpu: &WGPUContext, enable: bool) {
        self.surface_config.present_mode = match enable {
            true => wgpu::PresentMode::AutoVsync,
            false => wgpu::PresentMode::AutoNoVsync,
        };
        self.surface.configure(&wgpu.device, &self.surface_config);
    }

    pub fn vsync_enabled(&self) -> bool {
        use wgpu::PresentMode::*;
        match self.surface_config.present_mode {
            AutoVsync | Fifo | FifoRelaxed => true,
            AutoNoVsync | Mailbox | Immediate => false,
        }
    }

    pub fn get_texture(&self, wgpu: &WGPUContext) -> wgpu::SurfaceTexture {
        match self.surface.get_current_texture() {
            Err(_) => {
                // Surface lost or something so reconfigure and try to reobtain
                self.surface.configure(&wgpu.device, &self.surface_config);

                self.surface
                    .get_current_texture()
                    .expect("failed to get surface texture")
            }
            Ok(output) => output,
        }
    }
}
