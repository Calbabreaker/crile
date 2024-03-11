mod tabs;

pub use crate::tabs::{EditorState, Selection};

pub struct CrileEditorApp {
    egui: crile_egui::EguiContext,
    state: EditorState,
}

impl crile::Application for CrileEditorApp {
    fn new(engine: &mut crile::Engine) -> Self {
        Self {
            egui: crile_egui::EguiContext::new(engine),
            state: EditorState::default(),
        }
    }

    fn update(&mut self, engine: &mut crile::Engine) {
        tabs::viewport::check_texture(&mut self.state, &engine.gfx.wgpu, &mut self.egui);

        let ctx = self.egui.begin_frame(engine);

        egui::TopBottomPanel::top("top_panel").show(&ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        engine.request_exit();
                    }
                });
            });
        });

        egui::SidePanel::left("Hierachy")
            .width_range(150.0..=300.0)
            .show(&ctx, |ui| {
                tabs::hierarchy::show(&mut self.state, ui);
            });

        egui::SidePanel::right("Inspector")
            .width_range(260.0..=500.0)
            .show(&ctx, |ui| {
                tabs::inspector::show(&mut self.state, ui);
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(&ctx, |ui| {
                tabs::viewport::show(&mut self.state, ui);
            });

        self.egui.end_frame(engine, ctx);
    }

    fn render(&mut self, engine: &mut crile::Engine) {
        // Render to the viewport texture to be displayed in the viewport panel
        if let Some(texture) = &self.state.viewport_texture {
            let mut scene_render_pass = crile::RenderPass::new(
                &mut engine.gfx,
                Some(crile::Color::BLACK),
                self.state.depth_texture.as_ref(),
                Some(texture.view()),
            );

            self.state.scene.render(&mut scene_render_pass);
        }

        // Now render onto the window
        let mut render_pass =
            crile::RenderPass::new(&mut engine.gfx, Some(crile::Color::BLACK), None, None);
        self.egui.render(&mut render_pass);
    }

    fn event(&mut self, engine: &mut crile::Engine, event: &crile::Event) {
        if event.kind == crile::EventKind::WindowClose {
            engine.request_exit();
        }

        if event.window_id == Some(engine.window.id()) {
            self.egui.process_event(engine, &event.kind);
        }
    }
}

fn main() {
    crile::run_app::<CrileEditorApp>().unwrap();
}
