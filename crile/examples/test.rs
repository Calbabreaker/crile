#[derive(Default)]
pub struct TestApp {}

impl crile::Application for TestApp {
    fn init(&mut self, engine: &mut crile::Engine) {}
    fn update(&mut self, engine: &mut crile::Engine) {}
    fn render(&mut self, engine: &mut crile::Engine) {}

    fn event(&mut self, engine: &mut crile::Engine, event: &crile::Event) {
        match event {
            crile::Event::WindowClose => engine.request_close(),
            _ => (),
        }
    }
}

fn main() {
    env_logger::init();
    crile::run(TestApp::default())
}
