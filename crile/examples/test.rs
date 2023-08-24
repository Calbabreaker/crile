#[derive(Default)]
pub struct TestApp {}

impl crile::Application for TestApp {
    fn init(&mut self, engine: &mut crile::Engine) {
        engine.renderer.api.set_vsync(true);
    }

    fn update(&mut self, engine: &mut crile::Engine) {
        // if engine.input.key_just_pressed(crile::KeyCode::Space) {
        println!("Framerate: {}", engine.time.framerate());
        // }
    }

    fn render(&mut self, engine: &mut crile::Engine, instance: &mut crile::RenderInstance) {
        engine.renderer.render(instance);
    }

    fn event(&mut self, engine: &mut crile::Engine, event: &crile::Event) {
        match event {
            crile::Event::WindowClose => engine.request_close(),
            _ => (),
        }
    }
}

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Warn)
        .init();
    crile::run(TestApp::default()).unwrap_or_else(|error| log::error!("{error}"));
}
