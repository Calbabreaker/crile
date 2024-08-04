#![allow(unused)]

pub struct SceneApp {
    scene: crile::Scene,
}

#[derive(Clone, Default, Debug)]
struct TestComponent;

impl crile::Application for SceneApp {
    fn new(engine: &mut crile::Engine) -> Self {
        let mut scene = crile::Scene::default();
        scene.world.spawn((
            crile::TransformComponent::default(),
            crile::SpriteComponent {
                color: crile::Color::from_srgba(255, 0, 0, 255),
                ..Default::default()
            },
        ));

        let index = scene.world.spawn((TestComponent,));
        let component = scene.world.get::<TestComponent>(index);

        scene.set_viewport(engine.main_window().size().as_vec2());
        Self { scene }
    }

    fn update(&mut self, engine: &mut crile::Engine, event_loop: &crile::ActiveEventLoop) {
        if engine.time.frame_count() % 300 == 0 {
            dbg!(engine.time.frame_rate());
        }
    }

    fn render(&mut self, engine: &mut crile::Engine) {
        let mut render_pass =
            crile::RenderPass::new(&mut engine.gfx, Some(crile::Color::BLACK), None, None);
        self.scene.render(&mut render_pass, glam::Mat4::IDENTITY);
    }

    fn event(&mut self, engine: &mut crile::Engine, event: crile::Event) {
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
