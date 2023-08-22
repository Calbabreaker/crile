use crate::{
    events::{process_event, Event},
    renderer::{Renderer, RendererAPI},
    window::Window,
};

pub trait Application {
    fn init(&mut self, engine: &mut Engine);
    fn update(&mut self, engine: &mut Engine);
    fn render(&mut self, engine: &mut Engine);
    fn event(&mut self, engine: &mut Engine, event: &Event);
}

pub struct Engine {
    renderer_api: RendererAPI,
    renderer: Renderer,
    window: Window,
    should_close: bool,
}

impl Engine {
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Self {
        let window = Window::new(&event_loop);
        let renderer_api = pollster::block_on(RendererAPI::new(&window));
        Self {
            renderer: Renderer::new(&renderer_api),
            renderer_api,
            window,
            should_close: false,
        }
    }

    fn update(&mut self, app: &mut impl Application) {
        app.update(self);
        self.window.request_redraw();
    }

    fn render(&mut self, app: &mut impl Application) {
        match self.renderer_api.begin_frame() {
            Ok(mut instance) => {
                app.render(self);
                self.renderer.render(&mut instance, &self.renderer_api);
                self.renderer_api.present_frame(instance);
            }
            Err(wgpu::SurfaceError::Lost) => self.renderer_api.resize(self.window.size()),
            Err(wgpu::SurfaceError::OutOfMemory) => panic!("GPU out of memory"),
            Err(e) => log::error!("{:?}", e),
        };
    }

    pub fn event(&mut self, app: &mut impl Application, event: &Event) {
        match event {
            Event::WindowResize { size } => self.renderer_api.resize(*size),
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
                engine.update(&mut app);
            }
            winit::event::Event::RedrawRequested(_) => {
                engine.render(&mut app);
            }
            winit::event::Event::NewEvents(_) => (),
            winit::event::Event::RedrawEventsCleared => (),
            event => engine.event(&mut app, &process_event(event)),
        };

        if engine.should_close {
            *control_flow = winit::event_loop::ControlFlow::Exit;
        }
    });
}
