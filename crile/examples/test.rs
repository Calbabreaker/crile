#[derive(Default)]
pub struct TestApp {}

impl crile::Application for TestApp {
    fn init(&mut self, engine: &mut crile::Engine) {}
    fn update(&mut self, engine: &mut crile::Engine) {
        println!("Framerate: {}", engine.time.frame_rate());
    }

    fn render(&mut self, engine: &mut crile::Engine) {
        if let Some(mut instance) = engine.renderer.api.begin_frame() {
            engine.renderer.render(&mut instance);
            engine.renderer.api.present_frame(instance);
        }
    }

    fn event(&mut self, engine: &mut crile::Engine, event: &crile::Event) {
        // dbg!(event);
        match event {
            crile::Event::WindowClose => engine.request_close(),
            _ => (),
        }
    }
}

fn main() {
    env_logger::init();
    crile::run(TestApp::default());
}
