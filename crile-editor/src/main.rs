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
                    .resizable(false)
                    .show(&ctx, |ui| {
                        egui::Resize::default().with_stroke(false).show(ui, |ui| {
                            if self.state.preferences.show(ui) {
                                self.apply_preferences(engine);
                            }
                        });
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
                sections::top_panel::show(ui, &mut self.state);
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
                    if response.hovered() && self.state.scene_runtime.is_none() {
                        ui.input(|i| {
                            self.state
                                .editor_camera
                                .process_input(i, &self.state.preferences)
                        });
                    }
                }
            });

        self.egui.end_frame(engine, ctx);

        sections::inspector::update_assets(&mut self.state, engine);
        if let Some(scene_runtime) = self.state.scene_runtime.as_mut() {
            if let Err(err) = scene_runtime.update() {
                log::error!("{err}");
                self.state.stop_scene();
            }
        }
    }

    fn render(&mut self, engine: &mut crile::Engine) {
        // First render onto the viewport texture which will be put in an egui panel
        let viewport_size = self.state.editor_view.size.as_vec2();
        self.state.active_scene().set_viewport(viewport_size);
        self.state.editor_camera.set_viewport(viewport_size);

        if let Some(mut render_pass) = self.state.editor_view.get_render_pass(engine) {
            if let Some(scene_runtime) = self.state.scene_runtime.as_mut() {
                scene_runtime.render(&mut render_pass);
            } else {
                self.state
                    .scene
                    .render(&mut render_pass, self.state.editor_camera.view_projection());
            }
        }

        // Now render onto the window
        let mut render_pass = crile::RenderPass::new(&mut engine.gfx, None, None, None);
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
