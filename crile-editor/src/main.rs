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

    fn update(&mut self, engine: &mut crile::Engine, event_loop: &crile::ActiveEventLoop) {
        if engine
            .main_window()
            .input
            .key_just_pressed(crile::KeyCode::Tab)
        {
            engine.create_window(event_loop, crile::WindowAttributes::default());
        }

        self.state
            .editor_view
            .check_texture(&engine.gfx.wgpu, &mut self.egui);

        let ctx = self.egui.begin_frame(engine);
        let default_bg = ctx.style().visuals.noninteractive().bg_fill;

        let mut open = true;
        let popup = egui::Window::new("Popup")
            .default_pos(ctx.screen_rect().size().to_pos2() / 2.)
            .open(&mut open)
            .resizable(false);

        match self.state.window_open {
            WindowKind::Preferences => {
                popup.show(&ctx, |ui| {
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
                sections::top_panel::show(ui, &mut self.state, engine, event_loop);
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
                    if response.hovered() {
                        ui.input(|i| {
                            if let Some(pos) = i.pointer.latest_pos() {
                                self.state.editor_camera.mouse_position = glam::Vec2::new(
                                    pos.x - response.rect.left(),
                                    pos.y - response.rect.top(),
                                );
                            }
                            self.state
                                .editor_camera
                                .process_input(i, &self.state.preferences)
                        });
                    }
                }
            });

        self.egui.end_frame(engine, ctx);

        if engine
            .main_window()
            .input
            .mouse_just_pressed(crile::MouseButton::Left)
        {
            self.state.active_scene.spawn(
                "test",
                (
                    crile::TransformComponent {
                        translation: self
                            .state
                            .editor_camera
                            .camera
                            .screen_to_world(self.state.editor_camera.mouse_position)
                            .extend(1.),
                        scale: glam::Vec3::splat(100.),
                        ..Default::default()
                    },
                    crile::SpriteComponent::default(),
                ),
                None,
            );
        }

        sections::inspector::update_assets(&mut self.state, engine);
        if let SceneState::Running(data) = &mut self.state.scene_state {
            if let Err(err) = data.scene_runner.update() {
                // Stop running if error
                log::error!("{err}");
                self.state.stop_scene(engine);
            }
        }
    }

    fn render(&mut self, engine: &mut crile::Engine) {
        // Check if the main window is rendering
        if engine.gfx.target_window_id() == Some(engine.main_window().id()) {
            // First render onto the viewport texture which will be put in an egui panel
            let viewport_size = self.state.editor_view.size.as_vec2();
            self.state.editor_camera.set_viewport(viewport_size);
            self.state.active_scene.set_viewport(viewport_size);
            if let Some(mut render_pass) = self.state.editor_view.get_render_pass(engine) {
                self.state.editor_camera.update();
                self.state
                    .active_scene
                    .render(&mut render_pass, self.state.editor_camera.view_projection());
            }

            // Now render onto the window
            let mut render_pass = crile::RenderPass::new(&mut engine.gfx, None, None, None);
            self.egui.render(&mut render_pass);
            return;
        }

        // Now this could only be the game window from here
        if let SceneState::Running(data) = &mut self.state.scene_state {
            // Render directly onto the game window
            let viewport_size = engine.get_window(data.game_window_id).unwrap().size();
            self.state
                .active_scene
                .set_viewport(viewport_size.as_vec2());
            let mut render_pass = crile::RenderPass::new(&mut engine.gfx, None, None, None);
            data.scene_runner
                .render(&mut render_pass, &mut self.state.active_scene);
        }
    }

    fn event(&mut self, engine: &mut crile::Engine, event: crile::Event) {
        if event.kind == crile::EventKind::WindowClose {
            if event.window_id == Some(engine.main_window().id()) {
                engine.request_exit();
            } else {
                self.state.stop_scene(engine)
            }
        }

        self.egui.process_event(engine, &event);
    }
}

impl CrileEditorApp {
    pub fn apply_preferences(&mut self, engine: &mut crile::Engine) {
        self.egui
            .set_ui_scale(self.state.preferences.ui_scale, engine.main_window().size());
    }
}

fn main() {
    env_logger::builder()
        .filter_module("crile", log::LevelFilter::Trace)
        .filter_level(log::LevelFilter::Warn)
        .init();

    crile::run_app::<CrileEditorApp>().unwrap();
}
