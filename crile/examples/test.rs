#![allow(unused)]

#[derive(Default)]
pub struct TestApp {
    batch: crile::BatchData,
}

impl crile::Application for TestApp {
    fn init(&mut self, engine: &mut crile::Engine) {
        engine.camera.ortho_size = 10.0;
        self.batch.textures.push(crile::Texture::new(
            &engine.renderer_api,
            1,
            1,
            &[255, 255, 255, 255],
        ));
    }

    fn update(&mut self, engine: &mut crile::Engine) {
        // if engine.input.key_just_pressed(crile::KeyCode::Space) {
        // println!("Framerate: {}", engine.time.framerate());
        // }
    }

    fn render(&mut self, engine: &mut crile::Engine, instance: &mut crile::RenderInstance) {
        engine
            .renderer_2d
            .begin(&engine.renderer_api, &engine.camera);

        let rows = 10;
        let cols = 10;
        self.batch.sprites = (0..rows * cols)
            .map(|i| {
                let position = crile::Vector3::new((i % cols) as f32, (i / rows) as f32, 0.0);
                crile::SpriteData {
                    texture_index: 0,
                    transform: crile::Matrix4::from_translation(position),
                    color: crile::Color::from_hex(0xffffff),
                }
            })
            .collect::<Vec<_>>();

        engine
            .renderer_2d
            .draw_batch(&engine.renderer_api, instance, &self.batch);
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
        .filter_module("crile", log::LevelFilter::Trace)
        .filter_level(log::LevelFilter::Error)
        .init();
    crile::run(TestApp::default()).unwrap_or_else(|error| log::error!("{error}"));
}
