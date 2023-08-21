#[derive(Default)]
pub struct TestApp {}

impl crile::Application for TestApp {
    fn init(&mut self, engine: &mut crile::Engine) {}

    fn update(&mut self, engine: &mut crile::Engine) {
        println!("RUnning")
    }
    fn render(&mut self, engine: &mut crile::Engine) {}
}

fn main() {
    crile::run(TestApp::default())
}
