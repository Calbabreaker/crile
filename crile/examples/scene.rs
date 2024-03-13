#![allow(unused)]

pub struct SceneApp {
    scene: crile::Scene,
}

impl crile::Application for SceneApp {
    fn new(engine: &mut crile::Engine) -> Self {
        let mut scene = crile::Scene::default();
        scene.world.spawn((crile::TransformComponent::default(),));
        scene.world.spawn((
            crile::TransformComponent::default(),
            crile::CameraComponent::default(),
        ));

        scene.world.spawn((
            crile::TransformComponent::default(),
            crile::SpriteRendererComponent {
                color: crile::Color::from_rgb(255, 0, 0),
                ..Default::default()
            },
        ));

        scene.set_viewport(engine.window.size());
        Self { scene }
    }

    fn update(&mut self, engine: &mut crile::Engine) {}

    fn render(&mut self, engine: &mut crile::Engine) {
        let mut render_pass =
            crile::RenderPass::new(&mut engine.gfx, Some(crile::Color::BLACK), None, None);
        self.scene.render(&mut render_pass);
    }

    fn event(&mut self, engine: &mut crile::Engine, event: &crile::Event) {
        match event.kind {
            crile::EventKind::WindowClose => engine.request_exit(),
            crile::EventKind::WindowResize { size } => self.scene.set_viewport(size),
            _ => (),
        }
    }
}

fn main() {
    crile::run_app::<SceneApp>().unwrap()
}
