#![allow(unused)]

#[derive(Default)]
pub struct TestApp {
    camera: crile::Camera,
    textures: Vec<crile::Texture>,
    instances: Vec<crile::RenderInstance>,
    egui: crile::egui::EguiContext,
    visibile: bool,
    text: String,
    world: crile::World,
}

impl crile::Application for TestApp {
    fn init(&mut self, engine: &mut crile::Engine) {
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
                crile::RenderInstance {
                    transform: glam::Mat4::from_translation(position),
                    color: crile::Color::WHITE,
                }
            })
            .collect();
        self.egui.init(engine);

        let ida = self.world.spawn(());
        self.world.spawn((crile::TransformComponent::default(),));
        self.world.despawn(ida);
        let id = self.world.spawn((crile::TransformComponent::default(),));
        let mut a = self.world.get_entity(id);
        dbg!(a.get::<crile::TransformComponent>());
        a.add(crile::SpriteRendererComponent::default());
        a.remove::<crile::SpriteRendererComponent>();
        a.add(crile::SpriteRendererComponent::default());
        assert!(id == ida);
        dbg!(id);
    }

    fn update(&mut self, engine: &mut crile::Engine) {
        // if engine.input.key_just_pressed(crile::KeyCode::Space) {
        // println!("Framerate: {}", engine.time.framerate());
        // }
        //
        //

        self.egui.update(engine, |ctx, engine| {
            egui::Window::new("hello").show(ctx, |ui| {
                ui.label("Hello world!");
                ui.checkbox(&mut self.visibile, "Click me");

                if self.visibile {
                    ui.text_edit_singleline(&mut self.text);
                }

                for (transform, sprite) in self
                    .world
                    .query::<(crile::TransformComponent, crile::SpriteRendererComponent)>()
                {
                    ui.label(format!("{transform:?} {sprite:?}"));
                }
            });
        });
    }

    fn render(&mut self, engine: &mut crile::Engine) {
        let mut render_pass =
            crile::RenderPass::new(&mut engine.gfx, Some(crile::Color::BLACK), None);
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

        self.egui.render(&mut render_pass);
    }

    fn event(&mut self, engine: &mut crile::Engine, event: &crile::Event) {
        match event {
            crile::Event::WindowClose => engine.request_close(),
            crile::Event::WindowResize { size } => self.camera.resize(size.as_vec2()),
            _ => (),
        }

        self.egui.event(engine, event);
    }
}

fn main() {
    crile::run(TestApp::default())
}
