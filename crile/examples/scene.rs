#![allow(unused)]

#[derive(Default)]
pub struct SceneApp {
    scene: crile::Scene,
}

impl crile::Application for SceneApp {
    fn init(&mut self, engine: &mut crile::Engine) {
        self.scene
            .world
            .spawn((crile::TransformComponent::default(),));
        self.scene.world.spawn((
            crile::TransformComponent::default(),
            crile::CameraComponent::default(),
        ));

        self.scene.world.spawn((
            crile::TransformComponent::default(),
            crile::SpriteRendererComponent {
                color: crile::Color::from_rgb(255, 0, 0),
            },
        ));

        self.scene.set_viewport(engine.window.size().as_vec2());
    }

    fn update(&mut self, engine: &mut crile::Engine) {}

    fn render(&mut self, engine: &mut crile::Engine) {
        let mut render_pass =
            crile::RenderPass::new(&mut engine.gfx, Some(crile::Color::BLACK), None);
        self.scene.render(&mut render_pass);
    }

    fn event(&mut self, engine: &mut crile::Engine, event: &crile::Event) {
        match event {
            crile::Event::WindowClose => engine.request_exit(),
            crile::Event::WindowResize { size } => self.scene.set_viewport(size.as_vec2()),
            _ => (),
        }
    }
}

fn main() {
    crile::run_app(SceneApp::default()).unwrap()
}
