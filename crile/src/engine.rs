use crate::{
    events::{convert_event, Event},
    graphics::Renderer,
    input::Input,
    time::Time,
    window::Window,
};

pub trait Application {
    fn init(&mut self, engine: &mut Engine);
    fn update(&mut self, engine: &mut Engine);
    fn render(&mut self, engine: &mut Engine);
    fn event(&mut self, engine: &mut Engine, event: &Event);
}

pub struct Engine {
    pub renderer: Renderer,
    pub window: Window,
    pub time: Time,
    pub input: Input,
    should_close: bool,
}

impl Engine {
    fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Self {
        let window = Window::new(event_loop);
        Self {
            renderer: pollster::block_on(Renderer::new(&window)),
            time: Time::new(),
            input: Input::default(),
            window,
            should_close: false,
        }
    }

    fn update(&mut self, app: &mut impl Application) {
        self.time.update();
        app.update(self);
        app.render(self);
        self.input.clear();
    }

    fn event(&mut self, app: &mut impl Application, event: &Event) {
        match event {
            Event::ApplicationUpdate => {
                self.update(app);
                return;
            }
            Event::WindowResize { size } => self.renderer.api.resize(*size),
            _ => (),
        };

        self.input.process_event(event);
        app.event(self, event);
    }

    pub fn request_close(&mut self) {
        self.should_close = true;
    }
}

pub fn run(mut app: impl Application + 'static) -> Result<(), crate::Error> {
    let event_loop = winit::event_loop::EventLoop::new()?;
    let mut engine = Engine::new(&event_loop);
    app.init(&mut engine);

    event_loop.run(move |event, _, control_flow| {
        if let Some(event) = convert_event(event) {
            engine.event(&mut app, &event);
        }

        if engine.should_close {
            control_flow.set_exit()
        }
    })?;
    Ok(())
}
