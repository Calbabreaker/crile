use crate::window::WindowSystem;

pub struct Engine {
    pub window: WindowSystem,
}

impl Engine {
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Self {
        Self {
            window: WindowSystem::new(&event_loop),
        }
    }
}

pub trait Application {
    #[allow(unused)]
    fn init(&mut self, engine: &mut Engine) {}
    fn update(&mut self, engine: &mut Engine);
    fn render(&mut self, engine: &mut Engine);
}

pub fn run(mut app: impl Application + 'static) {
    let event_loop = winit::event_loop::EventLoop::new();
    let mut engine = Engine::new(&event_loop);

    event_loop.run(move |event, _, control_flow| match event {
        winit::event::Event::MainEventsCleared => app.update(&mut engine),
        winit::event::Event::RedrawRequested(_) => app.render(&mut engine),
        _ => (),
    })
}
