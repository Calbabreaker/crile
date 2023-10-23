use crate::scene_hierachy_panel::SceneHierachyPanel;

mod scene_hierachy_panel;

#[derive(Default)]
pub struct SceneApp {
    egui: crile::EguiContext,
    scene: crile::Scene,
    scene_hierachy_panel: SceneHierachyPanel,
}

impl crile::Application for SceneApp {
    fn init(&mut self, engine: &mut crile::Engine) {
        self.scene.world.spawn((
            crile::IdentifierComponent {
                name: "Camera".to_string(),
            },
            crile::TransformComponent::default(),
            crile::CameraComponent::default(),
        ));

        self.scene.world.spawn((
            crile::IdentifierComponent {
                name: "Sprite".to_string(),
            },
            crile::TransformComponent::default(),
            crile::SpriteRendererComponent {
                color: crile::Color::from_rgb(255, 0, 0),
            },
        ));

        self.scene.resize(engine.window.size().as_vec2());
    }

    fn update(&mut self, engine: &mut crile::Engine) {
        self.egui.update(engine, |ctx, engine| {
            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                // The top panel is often a good place for a menu bar:
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            engine.request_close();
                        }
                    });
                });
            });

            self.scene_hierachy_panel.show_scene(ctx, &mut self.scene);
        });
    }

    fn render(&mut self, engine: &mut crile::Engine) {
        let mut render_pass =
            crile::RenderPass::new(&mut engine.gfx, Some(crile::Color::BLACK), None);
        self.scene.render(&mut render_pass);
        self.egui.render(&mut render_pass);
    }

    fn event(&mut self, engine: &mut crile::Engine, event: &crile::Event) {
        match event {
            crile::Event::WindowClose => engine.request_close(),
            crile::Event::WindowResize { size } => self.scene.resize(size.as_vec2()),
            _ => (),
        }

        self.egui.event(engine, event);
    }
}

fn main() {
    crile::run(SceneApp::default())
}
