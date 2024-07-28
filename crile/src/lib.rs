mod asset;
mod clipboard;
mod ecs;
mod engine;
mod events;
mod graphics;
mod ref_id;
mod scene;
mod scripting;
mod time;
mod utils;

pub use asset::*;
pub use clipboard::*;
pub use ecs::*;
pub use engine::*;
pub use events::*;
pub use graphics::*;
pub use ref_id::RefId;
pub use scene::*;
pub use scripting::*;
pub use time::*;
pub use utils::*;

#[cfg(test)]
mod tests {
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
                engine.gfx.wgpu.set_vsync(true, engine.main_window().id());
                Self { scene }
            }

            fn update(&mut self, engine: &mut Engine, _event_loop: &ActiveEventLoop) {
                if engine.time.elapsed() > Duration::from_secs(1) {
                    engine.request_exit();
                }
            }

            fn render(&mut self, engine: &mut Engine) {
                let mut render_pass =
                    RenderPass::new(&mut engine.gfx, Some(Color::BLACK), None, None);
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
}
