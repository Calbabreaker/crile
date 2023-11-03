use copypasta::ClipboardProvider;

pub struct Window {
    window: winit::window::Window,
    clipboard: copypasta::ClipboardContext,
}

impl Window {
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Self {
        Self {
            window: winit::window::WindowBuilder::new()
                .with_title("Crile")
                .build(event_loop)
                .unwrap(),
            clipboard: copypasta::ClipboardContext::new().unwrap(),
        }
    }

    /// Returns the width and height of the window
    /// Guarentees to be at least 1
    pub fn size(&self) -> glam::UVec2 {
        let size = self.window.inner_size();
        glam::UVec2::new(size.width.max(1), size.height.max(1))
    }

    pub(crate) fn win(&self) -> &winit::window::Window {
        &self.window
    }

    pub fn set_clipboard(&mut self, contents: String) {
        self.clipboard.set_contents(contents).unwrap();
    }

    pub fn get_clipboard(&mut self) -> String {
        self.clipboard.get_contents().unwrap()
    }
}
