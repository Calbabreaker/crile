#![allow(unused)]

#[derive(Default)]
pub struct TestApp {
    camera: crile::Camera,
    textures: Vec<crile::Texture>,
}

impl crile::Application for TestApp {
    fn init(&mut self, engine: &mut crile::Engine) {
        self.camera.resize(engine.window.size().as_vec2());
        self.camera.ortho_size = 10.0;
        self.textures.push(crile::Texture::from_image(
            &engine.gfx.wgpu,
            image::open("assets/test.png").unwrap(),
        ));

        // self.batch.textures.push(crile::Texture::new(
        //     &engine.renderer_gfx,
        //     1,
        //     1,
        //     &[255, 255, 255, 255],
        // ));
        // let image = image::open("assets/test.png").unwrap();
        // self.batch
        //     .textures
        //     .push(crile::Texture::from_image(&engine.renderer_gfx, image))
    }

    fn update(&mut self, engine: &mut crile::Engine) {
        // if engine.input.key_just_pressed(crile::KeyCode::Space) {
        // println!("Framerate: {}", engine.time.framerate());
        // }
    }

    fn render(&mut self, engine: &mut crile::Engine) -> Result<(), crile::EngineError> {
        let mut render_pass = crile::RenderPass::new(&mut engine.gfx, Some(crile::Color::BLACK))?;

        // let rows = 10;
        // let cols = 10;
        // self.batch.sprites = (0..rows * cols)
        //     .map(|i| {
        //         let position = crile::Vector3::new((i % cols) as f32, (i / rows) as f32, 0.0);
        //         crile::SpriteData {
        //             texture_index: i % 2,
        //             transform: crile::Matrix4::from_translation(position),
        //             color: crile::Color::from_hex(0xffffff),
        //         }
        //     })
        //     .collect::<Vec<_>>();

        render_pass.draw_mesh_indexed(crile::DrawMeshParams {
            mesh: &render_pass.data.square_mesh,
            texture: &self.textures[0],
            uniform: crile::DrawUniform {
                transform: self.camera.get_projection()
                    * crile::Matrix4::from_translation(crile::Vector3::new(0., 0., 0.)),
            },
            shader: crile::RefId::clone(&render_pass.data.instance_shader),
        });

        Ok(())
    }

    fn event(&mut self, engine: &mut crile::Engine, event: &crile::Event) {
        match event {
            crile::Event::WindowClose => engine.request_close(),
            crile::Event::WindowResize { size } => self.camera.resize(size.as_vec2()),
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
