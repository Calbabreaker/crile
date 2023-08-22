use crate::{
    events::{process_event, Event},
    window::WindowSystem,
};

pub struct Engine {
    pub window: WindowSystem,
    should_close: bool,
}

impl Engine {
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Self {
        Self {
            window: WindowSystem::new(&event_loop),
            should_close: false,
        }
    }

    pub fn request_close(&mut self) {
        self.should_close = true;
    }
}

pub trait Application {
    fn init(&mut self, engine: &mut Engine);
    fn update(&mut self, engine: &mut Engine);
    fn render(&mut self, engine: &mut Engine);
    fn event(&mut self, engine: &mut Engine, event: &Event);
}

pub fn run(mut app: impl Application + 'static) {
    let event_loop = winit::event_loop::EventLoop::new();
    let mut engine = Engine::new(&event_loop);

    event_loop.run(move |event, _, control_flow| {
        match event {
            winit::event::Event::MainEventsCleared => {
                app.update(&mut engine);
                engine.window.request_redraw();
            }
            winit::event::Event::RedrawRequested(_) => {
                app.render(&mut engine);
            }
            winit::event::Event::NewEvents(_) => (),
            winit::event::Event::RedrawEventsCleared => (),
            event => {
                let event = process_event(event);
                app.event(&mut engine, &event);
            }
        };

        if engine.should_close {
            *control_flow = winit::event_loop::ControlFlow::Exit;
        }
    });
}
