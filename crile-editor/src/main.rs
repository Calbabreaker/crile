mod editor_camera;
mod editor_state;
mod preferences;
mod project;
mod sections;

pub use crate::editor_state::*;
pub use crate::preferences::Preferences;

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
        if let Some(last_opened_project) = &app.state.preferences.last_opened_project {
            app.state.open_project(Some(last_opened_project.clone()));
        }

        app
    }

    fn update(&mut self, engine: &mut crile::Engine) {
        self.state
            .editor_view
            .check_texture(&engine.gfx.wgpu, &mut self.egui);

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
                sections::top_panel::show(ui, &mut self.state, engine);
            });

        egui::SidePanel::left("Hierachy")
            .width_range(150.0..=400.0)
            .show(&ctx, |ui| {
                sections::hierarchy::show(ui, &mut self.state);
            });

        egui::SidePanel::right("Inspector")
            .width_range(260.0..=500.0)
            .show(&ctx, |ui| {
                sections::inspector::show(ui, &mut self.state);
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(&ctx, |ui| {
                // TODO: have seperate game and editor view
                if let Some(response) = self.state.editor_view.show(ui) {
                    if response.hovered() && self.state.scene_state == SceneState::Editing {
                        ui.input(|i| self.state.editor_camera.process_input(i));
                    }
                }
            });

        self.egui.end_frame(engine, ctx);

        sections::inspector::update_assets(&mut self.state, engine);
        if self.state.scene_state == SceneState::Running {
            self.state.scene.update_runtime(engine);
        }
    }

    fn render(&mut self, engine: &mut crile::Engine) {
        if let Some(mut render_pass) = self.state.editor_view.get_render_pass(engine) {
            let size = self.state.editor_view.size;
            self.state.scene.set_viewport(self.state.editor_view.size);
            self.state.editor_camera.set_viewport(size.as_vec2());

            if self.state.scene_state == SceneState::Editing {
                self.state
                    .scene
                    .render(&mut render_pass, self.state.editor_camera.view_projection());
            } else {
                self.state.scene.render_runtime(&mut render_pass);
            }
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
            .set_ui_scale(self.state.preferences.ui_scale, engine.window.size());
    }
}

fn main() {
    crile::run_app::<CrileEditorApp>().unwrap();
}
