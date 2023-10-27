use crile::Event;

use crate::{inspector_panel::InspectorPanel, scene_hierachy_panel::SceneHierachyPanel};

mod inspector_panel;
mod scene_hierachy_panel;

pub struct SceneApp {
    egui: crile_egui::EguiContext,
    scene: crile::Scene,
    scene_hierachy_panel: SceneHierachyPanel,
    inspector_panel: InspectorPanel,
    scene_texture_output: Option<crile::RefId<crile::Texture>>,
    scene_texture_id: Option<egui::TextureId>,
}

impl crile::Application for SceneApp {
    fn new(engine: &mut crile::Engine) -> Self {
        let mut scene = crile::Scene::default();
        scene.world.spawn((
            crile::MetaDataComponent {
                name: "Camera".to_string(),
            },
            crile::TransformComponent::default(),
            crile::CameraComponent::default(),
        ));

        scene.world.spawn((
            crile::MetaDataComponent {
                name: "Sprite".to_string(),
            },
            crile::TransformComponent::default(),
            crile::SpriteRendererComponent {
                color: crile::Color::from_rgb(255, 0, 0),
            },
        ));

        scene.set_viewport(engine.window.size().as_vec2());
        Self {
            egui: crile_egui::EguiContext::new(engine),
            scene,
            scene_texture_output: None,
            scene_texture_id: None,
            scene_hierachy_panel: SceneHierachyPanel::default(),
            inspector_panel: InspectorPanel::default(),
        }
    }

    fn update(&mut self, engine: &mut crile::Engine) {
        let ctx = self.egui.begin_frame(engine);

        egui::TopBottomPanel::top("top_panel").show(&ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        engine.request_close();
                    }
                });
            });
        });

        self.scene_hierachy_panel
            .show_scene(&ctx, &mut self.scene, &mut self.inspector_panel);
        egui::CentralPanel::default()
            .frame(egui::Frame::default().inner_margin(egui::Vec2::ZERO))
            .show(&ctx, |ui| {
                let available_size =
                    glam::uvec2(ui.available_width() as u32, ui.available_height() as u32);

                if available_size.x == 0 || available_size.y == 0 {
                    return;
                }

                // If the viewport size is different from the texture output
                let texture_invalid = match &self.scene_texture_output {
                    None => true,
                    Some(ref texture) => texture.view().size() != available_size,
                };

                if texture_invalid {
                    if let Some(texture) = self.scene_texture_output.take() {
                        self.egui.unregister_texture(&texture)
                    }

                    let texture = crile::Texture::new_render_attach(
                        &engine.gfx.wgpu,
                        available_size.x,
                        available_size.y,
                    )
                    .into();

                    self.scene_texture_id = Some(self.egui.register_texture(&texture));
                    self.scene_texture_output = Some(texture);
                    self.scene.set_viewport(available_size.as_vec2());
                }

                if let Some(id) = self.scene_texture_id {
                    ui.image(id, ui.available_size());
                }
            });

        self.egui.end_frame(engine, ctx);
    }

    fn render(&mut self, engine: &mut crile::Engine) {
        if let Some(texture) = &self.scene_texture_output {
            let mut scene_render_pass = crile::RenderPass::new(
                &mut engine.gfx,
                Some(crile::Color::BLACK),
                Some(texture.view()),
            );

            self.scene.render(&mut scene_render_pass);
        }

        let mut render_pass =
            crile::RenderPass::new(&mut engine.gfx, Some(crile::Color::BLACK), None);
        self.egui.render(&mut render_pass);
    }

    fn event(&mut self, engine: &mut crile::Engine, event: &Event) {
        if event == &Event::WindowClose {
            engine.request_close();
        }

        self.egui.event(engine, event);
    }
}

fn main() {
    crile::run::<SceneApp>().unwrap();
}
