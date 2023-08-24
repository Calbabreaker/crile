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

    pub fn size(&self) -> Vector2U {
        let size = self.window.inner_size();
        Vector2U::new(size.width, size.height)
    }

    pub(crate) fn handle(&self) -> &winit::window::Window {
        &self.window
    }
}
