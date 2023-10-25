#![allow(unused)]

pub struct TestApp {
    camera: crile::Camera,
    textures: Vec<crile::Texture>,
    instances: Vec<crile::RenderInstance>,
    egui: crile::EguiContext,
    visibile: bool,
    text: String,
    world: crile::World,
}

impl crile::Application for TestApp {
    fn new(engine: &mut crile::Engine) -> Self {
        let mut texture =
            crile::Texture::from_image(&engine.gfx.wgpu, image::open("assets/test.png").unwrap());
        texture.sampler_config = crile::SamplerConfig::nearest();

        let rows = 100;
        let cols = 100;
        let instances = (0..rows * cols)
            .map(|i| {
                let position = glam::Vec3::new((i % cols) as f32, (i / rows) as f32, 0.0);
                crile::RenderInstance {
                    transform: glam::Mat4::from_translation(position),
                    color: crile::Color::WHITE,
                }
            })
            .collect();

        let mut world = crile::World::default();
        let id = world.spawn((crile::TransformComponent::default(),));
        let mut a = world.entity(id);
        dbg!(a.get::<crile::TransformComponent>());
        a.add(crile::CameraComponent::default());
        a.remove::<crile::CameraComponent>();
        a.add(crile::CameraComponent::default());
        dbg!(id);

        Self {
            camera: crile::Camera::new(engine.window.size().as_vec2()),
            textures: vec![texture],
            instances,
            egui: crile::EguiContext::new(engine),
            visibile: false,
            text: "hello".to_owned(),
            world: crile::World::default(),
        }
    }

    fn update(&mut self, engine: &mut crile::Engine) {
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
        render_pass.draw_mesh_instanced(render_pass.data.square_mesh.as_ref(), &self.instances);

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
    crile::run::<TestApp>().unwrap();
}
