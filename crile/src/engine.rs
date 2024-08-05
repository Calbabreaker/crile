use std::collections::BTreeMap;

use crate::{
    AssetManager, Clipboard, Event, EventKind, GraphicsContext, Time, Window, WindowConfig,
    WindowId,
};
pub use winit::event_loop::ActiveEventLoop;

/// For applications to implement in order to run
#[allow(unused)]
pub trait Application {
    fn new(engine: &mut Engine) -> Self;
    fn update(&mut self, engine: &mut Engine, event_loop: &ActiveEventLoop);
    fn fixed_update(&mut self, engine: &mut Engine) {}
    fn render(&mut self, engine: &mut Engine);
    fn event(&mut self, engine: &mut Engine, event: Event);
    fn main_window_config() -> WindowConfig {
        WindowConfig::default()
    }
}

pub struct Engine {
    pub gfx: GraphicsContext,
    pub time: Time,
    pub asset_manager: AssetManager,
    pub clipboard: Clipboard,
    should_exit: bool,
    windows: BTreeMap<WindowId, Window>,
    main_window_id: WindowId,
}

impl Engine {
    /// Creates a new window using the active event loop
    pub fn create_window(
        &mut self,
        event_loop: &ActiveEventLoop,
        config: WindowConfig,
    ) -> WindowId {
        let window = Window::new(Window::new_winit(event_loop, config));
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
        let winit = Window::new_winit(event_loop, App::main_window_config());
        let gfx = GraphicsContext::new(&winit);

        Self {
            gfx,
            main_window_id: winit.id(),
            time: Time::default(),
            should_exit: false,
            clipboard: Clipboard::default(),
            asset_manager: AssetManager::default(),
            windows: BTreeMap::from([(winit.id(), Window::new(winit))]),
        }
    }

    fn render(&mut self, app: &mut impl Application, window_id: WindowId) {
        if let Some(window) = self.windows.get(&window_id) {
            self.gfx.begin_frame(window);
            app.render(self);
            let window = self.windows.get(&window_id).unwrap();
            window.winit.pre_present_notify();
            self.gfx.end_frame();
        }
    }

    fn update(&mut self, app: &mut impl Application, event_loop: &ActiveEventLoop) {
        self.time.wait_for_target_frame_rate();
        self.time.update();
        while self.time.should_call_fixed_update() {
            app.fixed_update(self);
        }

        app.update(self, event_loop);
        for window in &mut self.windows.values_mut() {
            window.input.clear();
            window.winit.request_redraw();
        }
    }

    fn event(&mut self, app: &mut impl Application, event: Event) {
        if let Some(window) = self.windows.get_mut(&event.window_id) {
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
    state: Option<(App, Engine)>,
}

impl<App: Application> Default for EngineRunner<App> {
    fn default() -> Self {
        Self { state: None }
    }
}

impl<App: Application> winit::application::ApplicationHandler<()> for EngineRunner<App> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut engine = Engine::new::<App>(event_loop);
        self.state = Some((App::new(&mut engine), engine));
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    }

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: winit::event::StartCause) {
        if cause != winit::event::StartCause::Poll {
            return;
        }

        if let Some((app, engine)) = self.state.as_mut() {
            engine.update(app, event_loop);
            if engine.should_exit {
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        _: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if let Some((app, engine)) = self.state.as_mut() {
            if event == winit::event::WindowEvent::RedrawRequested {
                engine.render(app, window_id);
                return;
            }

            if let Some(event) = Event::from_winit_window_event(window_id, event) {
                engine.event(app, event);
            }
        }
    }
}

pub fn run_app<App: Application>() -> Result<(), impl std::error::Error> {
    let event_loop = winit::event_loop::EventLoopBuilder::default().build()?;
    let mut app_runner = EngineRunner::<App>::default();
    event_loop.run_app(&mut app_runner)
}
