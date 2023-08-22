use crate::{
    events::{process_event, Event},
    graphics::Renderer,
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
    should_close: bool,
}

impl Engine {
    fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Self {
        let window = Window::new(&event_loop);
        Self {
            renderer: pollster::block_on(Renderer::new(&window)),
            window,
            should_close: false,
        }
    }

    fn event(&mut self, app: &mut impl Application, event: &Event) {
        match event {
            Event::WindowResize { size } => self.renderer.api.resize(*size),
            _ => (),
        };
        app.event(self, event);
    }

    pub fn request_close(&mut self) {
        self.should_close = true;
    }
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
            winit::event::Event::NewEvents(_) => {}
            winit::event::Event::RedrawEventsCleared => {}
            event => engine.event(&mut app, &process_event(event)),
        };

        if engine.should_close {
            *control_flow = winit::event_loop::ControlFlow::Exit;
        }
    });
}
