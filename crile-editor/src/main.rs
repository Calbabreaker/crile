mod options;
mod tabs;

pub use crate::{
    options::Options,
    tabs::{EditorState, Selection},
};

pub struct CrileEditorApp {
    egui: crile_egui::EguiContext,
    state: EditorState,
    options: Options,
    options_open: bool,
}

impl crile::Application for CrileEditorApp {
    fn new(engine: &mut crile::Engine) -> Self {
        Self {
            egui: crile_egui::EguiContext::new(engine),
            state: EditorState::default(),
            options: Options::default(),
            options_open: false,
        }
    }

    fn update(&mut self, engine: &mut crile::Engine) {
        tabs::viewport::check_texture(&mut self.state, &engine.gfx.wgpu, &mut self.egui);

        let ctx = self.egui.begin_frame(engine);

        egui::Window::new("Options")
            .default_pos(self.egui.actual_size().to_pos2() / 2.)
            .open(&mut self.options_open)
            .show(&ctx, |ui| {
                if self.options.show(ui) {
                    self.egui
                        .set_ui_scale(self.options.ui_scale, engine.window.size())
                }
            });

        egui::TopBottomPanel::top("top_panel").show(&ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        engine.request_exit();
                    }

                    if ui.button("Options").clicked() {
                        self.options_open = true;
                        ui.close_menu();
                    }
                });
            });
        });

        egui::SidePanel::left("Hierachy")
            .width_range(150.0..=400.0)
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

        self.state.scene.update(engine);
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
