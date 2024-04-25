use crate::{AssetManager, Event, EventKind, GraphicsContext, Input, Time, Window};
use copypasta::ClipboardProvider;

/// For applications to implement in order to run
pub trait Application {
    fn new(engine: &mut Engine) -> Self;
    fn update(&mut self, engine: &mut Engine);
    fn render(&mut self, engine: &mut Engine);
    fn event(&mut self, engine: &mut Engine, event: &Event);
}

pub struct Engine {
    pub gfx: GraphicsContext,
    pub window: Window,
    pub time: Time,
    pub input: Input,
    pub asset_manager: AssetManager,
    clipboard: copypasta::ClipboardContext,
    should_exit: bool,
}

impl Engine {
    fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Self {
        let gfx = GraphicsContext::new();
        let window = Window::new(&gfx.wgpu, event_loop);

        Self {
            gfx,
            asset_manager: Default::default(),
            time: Time::default(),
            input: Input::default(),
            window,
            should_exit: false,
            clipboard: copypasta::ClipboardContext::new().unwrap(),
        }
    }

    fn update(&mut self, app: &mut impl Application) {
        self.time.update();
        app.update(self);
        self.input.clear();
        self.window.winit.request_redraw();
    }

    fn render(&mut self, app: &mut impl Application) {
        self.gfx.begin_frame(&self.window);
        app.render(self);
        self.window.winit.pre_present_notify();
        self.gfx.end_frame();
    }

    fn event(&mut self, app: &mut impl Application, event: &Event) {
        match event.kind {
            EventKind::AppUpdate => self.update(app),
            EventKind::AppRedraw => self.render(app),
            _ => (),
        };

        if event.window_id == Some(self.window.id()) {
            self.window.process_event(&event.kind, &self.gfx.wgpu);
        }

        self.input.process_event(&event.kind);
        app.event(self, event);
    }

    pub fn request_exit(&mut self) {
        self.should_exit = true;
    }

    pub fn set_clipboard(&mut self, contents: String) {
        self.clipboard.set_contents(contents).unwrap();
    }

    pub fn get_clipboard(&mut self) -> String {
        self.clipboard.get_contents().unwrap()
    }
}

pub fn run_app<App: Application>() -> Result<(), winit::error::EventLoopError> {
    env_logger::builder()
        .filter_module("crile", log::LevelFilter::Trace)
        .filter_level(log::LevelFilter::Warn)
        .init();

    let event_loop = winit::event_loop::EventLoop::new()?;

    let mut engine = Engine::new(&event_loop);
    let mut app = App::new(&mut engine);

    event_loop.run(move |event, elwt| {
        if let Some(event) = crate::events::convert_event(event) {
            engine.event(&mut app, &event);
        }

        if engine.should_exit {
            elwt.exit()
        }
    })
}
