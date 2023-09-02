use crate::{
    events::{convert_event, Event},
    graphics::Renderer2D,
    input::Input,
    time::Time,
    window::Window,
    Camera, RenderInstance, RendererAPI,
};

pub trait Application {
    fn init(&mut self, engine: &mut Engine);
    fn update(&mut self, engine: &mut Engine);
    fn render(&mut self, engine: &mut Engine, instance: &mut RenderInstance);
    fn event(&mut self, engine: &mut Engine, event: &Event);
}

pub struct Engine {
    pub renderer_2d: Renderer2D,
    pub renderer_api: RendererAPI,
    pub window: Window,
    pub time: Time,
    pub input: Input,
    pub camera: Camera,
    should_close: bool,
}

impl Engine {
    fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Self {
        let window = Window::new(event_loop);
        let renderer_api = pollster::block_on(RendererAPI::new(&window));
        Self {
            renderer_2d: Renderer2D::new(&renderer_api),
            renderer_api,
            time: Time::default(),
            input: Input::default(),
            camera: Camera::new(window.size().as_vec2()),
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
        if let Some(mut instance) = self.renderer_api.begin_frame() {
            app.render(self, &mut instance);
            self.window.pre_present_notify();
            self.renderer_api.present_frame(instance);
        }
    }

    fn event(&mut self, app: &mut impl Application, event: &Event) {
        match event {
            Event::ApplicationUpdate => {
                self.update(app);
                return;
            }
            Event::ApplicationRender => {
                self.render(app);
                return;
            }
            Event::WindowResize { size } => {
                self.renderer_api.resize(*size);
                self.camera.resize(size.as_vec2());
            }
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
