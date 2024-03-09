mod tabs;

pub use crate::tabs::{EditorState, Selection};

pub struct CrileEditorApp {
    egui: crile_egui::EguiContext,
    state: EditorState,
    viewport_texture: Option<crile::RefId<crile::Texture>>,
}

impl crile::Application for CrileEditorApp {
    fn new(engine: &mut crile::Engine) -> Self {
        Self {
            egui: crile_egui::EguiContext::new(engine),
            state: EditorState::default(),
            viewport_texture: None,
        }
    }

    fn update(&mut self, engine: &mut crile::Engine) {
        self.check_scene_texture(&mut engine.gfx);

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

        egui::SidePanel::left("Hierachy").show(&ctx, |ui| {
            tabs::hierarchy::ui(&mut self.state, ui);
        });

        egui::SidePanel::right("Inspector").show(&ctx, |ui| {
            tabs::inspector::ui(&mut self.state, ui);
        });

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(&ctx, |ui| {
                tabs::viewport::ui(&mut self.state, ui);
            });

        self.egui.end_frame(engine, ctx);
    }

    fn render(&mut self, engine: &mut crile::Engine) {
        if let Some(texture) = &self.viewport_texture {
            let mut scene_render_pass = crile::RenderPass::new(
                &mut engine.gfx,
                Some(crile::Color::BLACK),
                Some(texture.view()),
            );

            self.state.scene.render(&mut scene_render_pass);
        }

        let mut render_pass =
            crile::RenderPass::new(&mut engine.gfx, Some(crile::Color::BLACK), None);
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
    pub fn check_scene_texture(&mut self, gfx: &mut crile::GraphicsContext) {
        if self.state.viewport_size.x == 0. || self.state.viewport_size.y == 0. {
            return;
        }

        // If the viewport size is different from the texture output
        let resized = match self.viewport_texture {
            None => true,
            Some(ref texture) => texture.view().size().as_vec2() != self.state.viewport_size,
        };

        if resized {
            if let Some(texture) = self.viewport_texture.take() {
                self.egui.unregister_texture(&texture);
            }

            let texture = crile::Texture::new_render_attach(
                &gfx.wgpu,
                self.state.viewport_size.x as u32,
                self.state.viewport_size.y as u32,
            )
            .into();

            self.state.viewport_texture_id = Some(self.egui.register_texture(&texture));
            self.viewport_texture = Some(texture);
            self.state.scene.set_viewport(self.state.viewport_size);
        }
    }
}

fn main() {
    crile::run_app::<CrileEditorApp>().unwrap();
}
