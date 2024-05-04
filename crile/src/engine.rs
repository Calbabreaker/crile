use std::sync::Arc;

use crate::{
    AssetManager, Clipboard, Event, EventKind, GraphicsContext, Time, Window, WindowAttributes,
    WindowId,
};
pub use winit::event_loop::ActiveEventLoop;

/// For applications to implement in order to run
pub trait Application {
    fn new(engine: &mut Engine) -> Self;
    fn update(&mut self, engine: &mut Engine, event_loop: &ActiveEventLoop);
    fn render(&mut self, engine: &mut Engine);
    fn event(&mut self, engine: &mut Engine, event: Event);
    fn main_window_attributes() -> WindowAttributes {
        WindowAttributes::default().with_title("Crile")
    }
}

pub struct Engine {
    pub gfx: GraphicsContext,
    pub time: Time,
    pub asset_manager: AssetManager,
    pub clipboard: Clipboard,
    should_exit: bool,
    windows: hashbrown::HashMap<WindowId, Window>,
    main_window_id: WindowId,
}

impl Engine {
    /// Creates a new window using the active event loop
    pub fn create_window(
        &mut self,
        event_loop: &ActiveEventLoop,
        attributes: WindowAttributes,
    ) -> WindowId {
        let window = Window::new(Window::new_winit(event_loop, attributes));
        self.gfx.wgpu.new_viewport(&window);

        let window_id = window.id();
        self.windows.insert(window_id, window);
        window_id
    }

    pub fn delete_window(&mut self, window_id: WindowId) {
        self.gfx.wgpu.delete_viewport(window_id);
        self.windows.remove(&window_id);
    }

    pub fn get_window(&self, window_id: WindowId) -> Option<&Window> {
        self.windows.get(&window_id)
    }

    pub fn request_exit(&mut self) {
        self.should_exit = true;
    }

    pub fn main_window(&self) -> &Window {
        self.get_window(self.main_window_id).unwrap()
    }

    fn new<App: Application>(event_loop: &ActiveEventLoop) -> Self {
        let winit = Window::new_winit(event_loop, App::main_window_attributes());
        let gfx = GraphicsContext::new(&winit);

        Self {
            gfx,
            main_window_id: winit.id(),
            time: Time::default(),
            should_exit: false,
            clipboard: Clipboard::default(),
            asset_manager: AssetManager::default(),
            windows: hashbrown::HashMap::from([(winit.id(), Window::new(winit))]),
        }
    }

    fn render(&mut self, app: &mut impl Application, window_id: WindowId) {
        if let Some(window) = self.windows.get(&window_id) {
            self.gfx.begin_frame(window);
            app.render(self);
        }
        if let Some(window) = self.windows.get(&window_id) {
            window.winit.pre_present_notify();
            self.gfx.end_frame();
        }
    }

    fn update(&mut self, app: &mut impl Application, event_loop: &ActiveEventLoop) {
        self.time.update();
        app.update(self, event_loop);
        for (_, window) in &mut self.windows {
            window.input.clear();
            window.winit.request_redraw();
        }
    }

    fn event(&mut self, app: &mut impl Application, event: Event) {
        if let Some(window) = event.window_id.and_then(|id| self.windows.get_mut(&id)) {
            window.input.process_event(&event.kind);

            if let EventKind::WindowResize { size } = event.kind {
                self.gfx.wgpu.resize_viewport(size, window.id())
            }
        }

        app.event(self, event);
    }
}

// This is used in order to run app and engine by implementing the winit app trait
struct EngineRunner<App: Application> {
    app: Option<App>,
    engine: Option<Engine>,
}

impl<App: Application> Default for EngineRunner<App> {
    fn default() -> Self {
        Self {
            app: None,
            engine: None,
        }
    }
}

impl<App: Application> winit::application::ApplicationHandler<()> for EngineRunner<App> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut engine = Engine::new::<App>(event_loop);
        self.app = Some(App::new(&mut engine));
        self.engine = Some(engine);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Engine should be valid here as it gets created before this function calls
        let engine = self.engine.as_mut().unwrap();
        engine.update(self.app.as_mut().unwrap(), event_loop);
        if engine.should_exit {
            event_loop.exit();
        }
    }

    fn window_event(
        &mut self,
        _: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let engine = self.engine.as_mut().unwrap();
        let app = self.app.as_mut().unwrap();

        if event == winit::event::WindowEvent::RedrawRequested {
            engine.render(app, window_id);
            return;
        }

        if let Some(event) = Event::from_winit_window_event(window_id, event) {
            engine.event(app, event);
        }
    }
}

pub fn run_app<App: Application>() -> Result<(), winit::error::EventLoopError> {
    let event_loop = winit::event_loop::EventLoop::new()?;
    let mut app_runner = EngineRunner::<App>::default();

    event_loop.run_app(&mut app_runner)
}
