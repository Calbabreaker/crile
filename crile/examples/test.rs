#![allow(unused)]

#[derive(Default)]
pub struct TestApp {
    camera: crile::Camera,
    textures: Vec<crile::Texture>,
    instances: Vec<crile::Instance>,
}

impl crile::Application for TestApp {
    fn init(&mut self, engine: &mut crile::Engine) {
        engine.gfx.set_vsync(false);
        self.camera.resize(engine.window.size().as_vec2());
        self.camera.ortho_size = 10.0;
        self.textures.push(crile::Texture::from_image(
            &engine.gfx.wgpu,
            image::open("assets/test.png").unwrap(),
        ));
        self.textures[0].sampler_config = crile::SamplerConfig::nearest();
        self.textures.push(engine.gfx.data.white_texture.clone());

        let rows = 100;
        let cols = 100;
        self.instances = (0..rows * cols)
            .map(|i| {
                let position = glam::Vec3::new((i % cols) as f32, (i / rows) as f32, 0.0);
                crile::Instance {
                    transform: glam::Mat4::from_translation(position),
                    color: crile::Color::WHITE,
                }
            })
            .collect();
    }

    fn update(&mut self, engine: &mut crile::Engine) {
        // if engine.input.key_just_pressed(crile::KeyCode::Space) {
        println!("Framerate: {}", engine.time.framerate());
        // }
    }

    fn render(&mut self, engine: &mut crile::Engine) {
        let mut render_pass = crile::RenderPass::new(&mut engine.gfx, Some(crile::Color::BLACK));

        render_pass.set_texture(&self.textures[0]);

        // Instanced version
        render_pass.set_shader(render_pass.data.instanced_shader.clone());
        render_pass.set_uniform(crile::DrawUniform {
            transform: self.camera.get_projection(),
        });
        render_pass.draw_mesh_instanced(&render_pass.data.square_mesh, &self.instances);

        // Single draw version
        // for instance in &self.instances {
        //     render_pass.set_uniform(crile::DrawUniform {
        //         transform: self.camera.get_projection() * instance.transform,
        //     });
        //     render_pass.draw_mesh_single(&render_pass.data.square_mesh);
        // }
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
    crile::run(TestApp::default())
}
