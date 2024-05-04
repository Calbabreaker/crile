use std::sync::Arc;

use crate::Input;
pub use winit::window::{CursorIcon, WindowAttributes, WindowId};

pub struct Window {
    // This needs to be Arc in order for WGPU to borrow it
    pub(crate) winit: Arc<winit::window::Window>,
    pub input: Input,
}

impl Window {
    pub(crate) fn new(winit: Arc<winit::window::Window>) -> Self {
        Self {
            winit,
            input: Input::default(),
        }
    }

    pub(crate) fn new_winit(
        event_loop: &winit::event_loop::ActiveEventLoop,
        attributes: WindowAttributes,
    ) -> Arc<winit::window::Window> {
        log::info!("Creating new window with title '{}'", attributes.title);
        Arc::new(
            event_loop
                .create_window(attributes)
                .expect("Failed to create window!"),
        )
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

    /// Sets the cursor icon
    /// Use None to make it invisible
    pub fn set_cursor_icon(&self, icon: Option<CursorIcon>) {
        if let Some(icon) = icon {
            self.winit.set_cursor_visible(true);
            self.winit.set_cursor(icon);
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
