mod editor_camera;
mod preferences;
mod project;
mod sections;

pub use crate::{
    preferences::Preferences,
    sections::{EditorState, Selection, WindowKind},
};

pub struct CrileEditorApp {
    egui: crile_egui::EguiContext,
    state: EditorState,
}

impl crile::Application for CrileEditorApp {
    fn new(engine: &mut crile::Engine) -> Self {
        let mut app = Self {
            egui: crile_egui::EguiContext::new(engine),
            state: EditorState::default(),
        };

        app.apply_preferences(engine);

        app
    }

    fn update(&mut self, engine: &mut crile::Engine) {
        sections::viewport::check_texture(&mut self.state, &engine.gfx.wgpu, &mut self.egui);

        let ctx = self.egui.begin_frame(engine);
        let default_bg = ctx.style().visuals.noninteractive().bg_fill;

        let mut open = true;
        match self.state.window_open {
            WindowKind::Preferences => {
                egui::Window::new("Preferences")
                    .default_pos(ctx.screen_rect().size().to_pos2() / 2.)
                    .open(&mut open)
                    .show(&ctx, |ui| {
                        if self.state.preferences.show(ui) {
                            self.apply_preferences(engine);
                        }
                    });
            }
            WindowKind::None => (),
        }

        if !open {
            self.state.window_open = WindowKind::None;
        }

        egui::TopBottomPanel::top("top_panel")
            .frame(egui::Frame::default().fill(default_bg).inner_margin(8.0))
            .show(&ctx, |ui| {
                sections::top_panel::show(&mut self.state, ui);
            });

        egui::SidePanel::left("Hierachy")
            .width_range(150.0..=400.0)
            .show(&ctx, |ui| {
                sections::hierarchy::show(&mut self.state, ui);
            });

        egui::SidePanel::right("Inspector")
            .width_range(260.0..=500.0)
            .show(&ctx, |ui| {
                sections::inspector::show(&mut self.state, ui);
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(&ctx, |ui| {
                sections::viewport::show(&mut self.state, ui);
            });

        self.egui.end_frame(engine, ctx);

        sections::inspector::update_assets(&mut self.state, engine);
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

            self.state.scene.render(
                &mut scene_render_pass,
                self.state.editor_camera.view_projection(),
            );
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

impl CrileEditorApp {
    pub fn apply_preferences(&mut self, engine: &mut crile::Engine) {
        self.egui
            .set_ui_scale(self.state.preferences.ui_scale, engine.window.size())
    }
}

fn main() {
    crile::run_app::<CrileEditorApp>().unwrap();
}
