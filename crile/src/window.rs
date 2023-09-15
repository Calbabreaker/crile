use crate::Vector2U;

pub struct Window {
    window: winit::window::Window,
}

impl Window {
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Self {
        Self {
            window: winit::window::WindowBuilder::new()
                .with_title("Opencuboids")
                .build(event_loop)
                .unwrap(),
        }
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    pub fn pre_present_notify(&self) {
        self.window.pre_present_notify();
    }

    /// Returns the width and height of the window
    /// Guarentees to be at least 1
    pub fn size(&self) -> Vector2U {
        let size = self.window.inner_size();
        Vector2U::new(size.width.max(1), size.height.max(1))
    }

    pub(crate) fn handle(&self) -> &winit::window::Window {
        &self.window
    }
}
