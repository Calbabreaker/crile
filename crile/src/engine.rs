use crate::{graphics::GraphicsContext, Event, Input, Time, Window};

/// For applications to implement in order to run
pub trait Application {
    fn init(&mut self, engine: &mut Engine);
    fn update(&mut self, engine: &mut Engine);
    fn render(&mut self, engine: &mut Engine);
    fn event(&mut self, engine: &mut Engine, event: &Event);
}

pub struct Engine {
    pub gfx: GraphicsContext,
    pub window: Window,
    pub time: Time,
    pub input: Input,
    should_exit: bool,
}

impl Engine {
    fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Self {
        let window = Window::new(event_loop);
        let gfx = GraphicsContext::new(&window);
        Self {
            gfx,
            time: Time::default(),
            input: Input::default(),
            window,
            should_exit: false,
        }
    }

    fn update(&mut self, app: &mut impl Application) {
        self.time.update();
        app.update(self);
        self.input.clear();
        self.window.win().request_redraw();
    }

    fn render(&mut self, app: &mut impl Application) {
        self.gfx.begin_frame();
        app.render(self);
        self.window.win().pre_present_notify();
        self.gfx.end_frame();
    }

    fn event(&mut self, app: &mut impl Application, event: &Event) {
        match event {
            Event::WindowResize { size } => self.gfx.resize(*size),
            Event::AppUpdate => self.update(app),
            Event::AppRedraw => self.render(app),
            _ => (),
        };

        self.input.process_event(event);
        app.event(self, event);
    }

    pub fn request_exit(&mut self) {
        self.should_exit = true;
    }
}

pub fn run_app(mut app: impl Application) -> Result<(), winit::error::EventLoopError> {
    env_logger::builder()
        .filter_module("crile", log::LevelFilter::Trace)
        .filter_level(log::LevelFilter::Warn)
        .init();

    let event_loop = winit::event_loop::EventLoop::new()?;

    let mut engine = Engine::new(&event_loop);
    app.init(&mut engine);

    event_loop.run(move |event, elwt| {
        if let Some(event) = crate::events::convert_event(event) {
            engine.event(&mut app, &event);
        }

        if engine.should_exit {
            elwt.exit()
        }
    })
}
