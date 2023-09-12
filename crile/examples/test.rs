#![allow(unused)]

#[derive(Default)]
pub struct TestApp {}

impl crile::Application for TestApp {
    fn init(&mut self, engine: &mut crile::Engine) {
        engine.camera.ortho_size = 10.0;
    }

    fn update(&mut self, engine: &mut crile::Engine) {
        // if engine.input.key_just_pressed(crile::KeyCode::Space) {
        println!("Framerate: {}", engine.time.framerate());
        // }
    }

    fn render(&mut self, engine: &mut crile::Engine, instance: &mut crile::RenderInstance) {
        engine
            .renderer_2d
            .begin(&engine.renderer_api, &engine.camera);

        for x in -10..10 {
            for y in -10..10 {
                let position = crile::Vector3::new(x as f32, y as f32, 0.0);
                engine.renderer_2d.draw_quad(
                    &crile::Matrix4::from_translation(position),
                    &crile::Color::from_hex(0xffffff),
                );
            }
        }

        engine.renderer_2d.flush(&engine.renderer_api, instance);
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
