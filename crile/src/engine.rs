use crate::{
    AssetManager, Clipboard, Event, GraphicsContext, Time, Window, WindowAttributes, WindowId,
};
pub use winit::event_loop::ActiveEventLoop;

/// For applications to implement in order to run
pub trait Application {
    fn new(engine: &mut Engine) -> Self;
    fn update(&mut self, engine: &mut Engine, event_loop: &ActiveEventLoop);
    fn render(&mut self, engine: &mut Engine);
    fn event(&mut self, engine: &mut Engine, event: Event);
}

#[derive(Default)]
pub struct Engine {
    pub gfx: GraphicsContext,
    pub time: Time,
    pub asset_manager: AssetManager,
    pub clipboard: Clipboard,
    should_exit: bool,
    windows: hashbrown::HashMap<WindowId, Window>,
    main_window_id: Option<WindowId>,
}

impl Engine {
    /// Creates a new window using the active event loop
    pub fn create_window(
        &mut self,
        event_loop: &ActiveEventLoop,
        attributes: WindowAttributes,
    ) -> WindowId {
        let window = Window::new(&self.gfx.wgpu, event_loop, attributes);
        let window_id = window.id();
        self.windows.insert(window_id, window);
        window_id
    }

    pub fn delete_window(&mut self, window_id: WindowId) {
        self.windows.remove(&window_id);
    }

    pub fn get_window(&self, window_id: WindowId) -> Option<&Window> {
        self.windows.get(&window_id)
    }

    pub fn request_exit(&mut self) {
        self.should_exit = true;
    }

    pub fn main_window(&self) -> &Window {
        // main_window_id gets created in new so this should never be none
        self.get_window(self.main_window_id.unwrap()).unwrap()
    }

    fn new(event_loop: &ActiveEventLoop) -> Self {
        let mut engine = Self::default();
        engine.main_window_id =
            Some(engine.create_window(event_loop, WindowAttributes::default().with_title("Crile")));
        engine
    }

    fn render(&mut self, app: &mut impl Application, window_id: WindowId) {
        if let Some(window) = self.windows.get(&window_id) {
            self.gfx.begin_frame(window);
            app.render(self);
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
            window.process_event(&event.kind, &self.gfx.wgpu);
        }

        app.event(self, event);
    }
}

struct AppRunner<App: Application> {
    app: Option<App>,
    engine: Option<Engine>,
}

impl<App: Application> Default for AppRunner<App> {
    fn default() -> Self {
        Self {
            app: None,
            engine: None,
        }
    }
}

impl<A: Application> winit::application::ApplicationHandler<()> for AppRunner<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut engine = Engine::new(event_loop);
        self.app = Some(A::new(&mut engine));
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
    env_logger::builder()
        .filter_module("crile", log::LevelFilter::Trace)
        .filter_level(log::LevelFilter::Warn)
        .init();

    let event_loop = winit::event_loop::EventLoop::new()?;
    let mut app_runner = AppRunner::<App>::default();

    event_loop.run_app(&mut app_runner)
}
