#![allow(unused)]

pub struct SceneApp {
    scene: crile::Scene,
}

#[derive(Clone, Default, Debug)]
struct TestComponent;

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
            crile::SpriteComponent {
                color: crile::Color::from_srgba(255, 0, 0, 255),
                ..Default::default()
            },
        ));

        let id = scene.world.spawn((TestComponent,));
        let component = scene.world.get::<TestComponent>(id);

        scene.set_viewport(engine.window.size().as_vec2());
        Self { scene }
    }

    fn update(&mut self, engine: &mut crile::Engine) {}

    fn render(&mut self, engine: &mut crile::Engine) {
        let mut render_pass =
            crile::RenderPass::new(&mut engine.gfx, Some(crile::Color::BLACK), None, None);
        self.scene.render_runtime(&mut render_pass);
    }

    fn event(&mut self, engine: &mut crile::Engine, event: &crile::Event) {
        match event.kind {
            crile::EventKind::WindowClose => engine.request_exit(),
            crile::EventKind::WindowResize { size } => self.scene.set_viewport(size.as_vec2()),
            _ => (),
        }
    }
}

fn main() {
    crile::run_app::<SceneApp>().unwrap()
}
