use std::sync::Arc;

use crate::{EventKind, WGPUContext, WindowId};

pub struct Window {
    pub(crate) winit: Arc<winit::window::Window>,
    pub viewport: Viewport,
}

impl Window {
    pub fn new(wgpu: &WGPUContext, event_loop: &winit::event_loop::EventLoop<()>) -> Self {
        let winit = Arc::new(
            winit::window::WindowBuilder::new()
                .with_title("Crile")
                .build(event_loop)
                .unwrap(),
        );

        Self {
            viewport: Viewport::new(wgpu, winit.clone()),
            winit,
        }
    }

    /// Returns the width and height of the window
    /// Guaranties to be at least 1
    pub fn size(&self) -> glam::UVec2 {
        let size = self.winit.inner_size();
        glam::UVec2::new(size.width.max(1), size.height.max(1))
    }

    pub fn id(&self) -> WindowId {
        self.winit.id()
    }

    pub fn process_event(&mut self, event: &EventKind, wgpu: &WGPUContext) {
        if let EventKind::WindowResize { size } = event {
            self.viewport.resize(*size, wgpu)
        };
    }
}

pub struct Viewport {
    pub(crate) surface: wgpu::Surface<'static>,
    pub(crate) surface_config: wgpu::SurfaceConfiguration,
}

impl Viewport {
    pub fn new_surface(wgpu: &WGPUContext, winit: Arc<winit::window::Window>) -> wgpu::Surface {
        wgpu.instance
            .create_surface(winit.clone())
            .expect("Failed to create surface!")
    }

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
