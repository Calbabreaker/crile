#![allow(unused)]

#[derive(Default)]
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
            },
        ));

        scene.set_viewport(engine.window.size().as_vec2());

        Self { scene }
    }

    fn update(&mut self, engine: &mut crile::Engine) {}

    fn render(&mut self, engine: &mut crile::Engine) {
        let mut render_pass =
            crile::RenderPass::new(&mut engine.gfx, Some(crile::Color::BLACK), None);
        self.scene.render(&mut render_pass);
    }

    fn event(&mut self, engine: &mut crile::Engine, event: &crile::Event) {
        match event {
            crile::Event::WindowClose => engine.request_close(),
            crile::Event::WindowResize { size } => self.scene.set_viewport(size.as_vec2()),
            _ => (),
        }
    }
}

fn main() {
    crile::run_app::<SceneApp>().unwrap()
}
