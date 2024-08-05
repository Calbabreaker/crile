use std::time::Duration;

use super::*;

#[test]
fn test() {
    pub struct Test {
        scene: Scene,
    }

    impl Application for Test {
        fn new(engine: &mut Engine) -> Self {
            let mut scene = Scene::default();
            scene.world.spawn((
                TransformComponent::default(),
                SpriteComponent {
                    color: Color::from_srgba(255, 0, 0, 255),
                    ..Default::default()
                },
            ));

            scene.set_viewport(engine.main_window().size().as_vec2());
            Self { scene }
        }

        fn update(&mut self, engine: &mut Engine, _event_loop: &ActiveEventLoop) {
            if engine.time.elapsed() > Duration::from_millis(250) {
                engine.request_exit();
            }
        }

        fn render(&mut self, engine: &mut Engine) {
            let mut render_pass = RenderPass::new(&mut engine.gfx, Some(Color::BLACK), None, None);
            self.scene.render(&mut render_pass, glam::Mat4::IDENTITY);
        }

        fn event(&mut self, _engine: &mut Engine, event: Event) {
            if let EventKind::WindowResize { size } = event.kind {
                self.scene.set_viewport(size.as_vec2())
            }
        }
    }

    run_app::<Test>().unwrap()
}
