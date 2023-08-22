use crate::{
    events::{process_event, Event},
    renderer::{MainRenderer, RenderInstance},
    window::Window,
};

pub struct Engine {
    pub main_renderer: MainRenderer,
    window: Window,
    should_close: bool,
}

impl Engine {
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Self {
        let window = Window::new(&event_loop);
        Self {
            main_renderer: pollster::block_on(MainRenderer::new(&window)),
            window,
            should_close: false,
        }
    }

    fn update(&mut self, app: &mut impl Application) {
        app.update(self);
        self.window.request_redraw();
    }

    fn render(&mut self, app: &mut impl Application) {
        match self.main_renderer.begin_frame() {
            Err(wgpu::SurfaceError::Lost) => self.main_renderer.resize(self.window.size()),
            Err(wgpu::SurfaceError::OutOfMemory) => panic!("GPU out of memory"),
            Err(e) => log::error!("{:?}", e),
            Ok(mut instance) => {
                self.main_renderer.begin_render_pass(&mut instance);
                self.main_renderer.present_frame(instance);

                app.render(self);
            }
        };
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
                engine.update(&mut app);
            }
            winit::event::Event::RedrawRequested(_) => {
                engine.render(&mut app);
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
