use crate::{input::Input, time::Time, window::Window, Event, GraphicsContext};

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
    should_close: bool,
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
            should_close: false,
        }
    }

    fn update(&mut self, app: &mut impl Application) {
        self.time.update();
        app.update(self);
        self.input.clear();
        self.window.request_redraw();
    }

    fn render(&mut self, app: &mut impl Application) {
        self.gfx.begin_frame();
        app.render(self);
        self.window.pre_present_notify();
        self.gfx.end_frame();
    }

    fn event(&mut self, app: &mut impl Application, event: &Event) {
        match event {
            Event::WindowResize { size } => self.gfx.resize(*size),
            _ => (),
        };

        self.input.process_event(event);
        app.event(self, event);
    }

    pub fn request_close(&mut self) {
        self.should_close = true;
    }
}

pub fn run(app: impl Application + 'static) {
    env_logger::builder()
        .filter_module("crile", log::LevelFilter::Trace)
        .filter_level(log::LevelFilter::Error)
        .init();

    if let Err(err) = try_run(app) {
        log::error!("{err}")
    }
}

fn try_run(mut app: impl Application + 'static) -> Result<(), winit::error::EventLoopError> {
    let event_loop = winit::event_loop::EventLoop::new()?;
    let mut engine = Engine::new(&event_loop);
    app.init(&mut engine);

    event_loop.run(move |event, _, control_flow| {
        match event {
            winit::event::Event::AboutToWait => engine.update(&mut app),
            winit::event::Event::RedrawRequested(_) => engine.render(&mut app),
            event => {
                if let Some(event) = crate::events::convert_event(event) {
                    engine.event(&mut app, &event);
                }
            }
        }

        if engine.should_close {
            control_flow.set_exit()
        }
    })
}
