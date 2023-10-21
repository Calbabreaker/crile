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

        self.scene.resize(engine.window.size().as_vec2());
    }

    fn update(&mut self, engine: &mut crile::Engine) {}

    fn render(&mut self, engine: &mut crile::Engine) {
        self.scene.render(&mut engine.gfx);
    }

    fn event(&mut self, engine: &mut crile::Engine, event: &crile::Event) {
        match event {
            crile::Event::WindowClose => engine.request_close(),
            crile::Event::WindowResize { size } => self.scene.resize(size.as_vec2()),
            _ => (),
        }
    }
}

fn main() {
    crile::run(SceneApp::default())
}
