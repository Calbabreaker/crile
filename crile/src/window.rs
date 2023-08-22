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

    pub fn size(&self) -> crate::Vector2 {
        let size = self.window.inner_size();
        crate::Vector2::new(size.width as f32, size.height as f32)
    }

    pub(crate) fn handle(&self) -> &winit::window::Window {
        &self.window
    }
}
