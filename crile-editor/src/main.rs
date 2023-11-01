use crate::tabs::{EditorState, Tab};

mod tabs;

pub struct CrileEditorApp {
    egui: crile_egui::EguiContext,
    dock_state: egui_dock::DockState<Tab>,
    state: EditorState,
    texture_output: Option<crile::RefId<crile::Texture>>,
}

impl crile::Application for CrileEditorApp {
    fn new(engine: &mut crile::Engine) -> Self {
        let mut dock_state = egui_dock::DockState::new(vec![Tab::Viewport]);

        let surface = dock_state.main_surface_mut();
        let [old, _] = surface.split_left(egui_dock::NodeIndex::root(), 0.15, vec![Tab::Hierarchy]);
        surface.split_right(old, 0.80, vec![Tab::Inspector]);

        Self {
            egui: crile_egui::EguiContext::new(engine),
            state: EditorState::default(),
            dock_state,
            texture_output: None,
        }
    }

    fn update(&mut self, engine: &mut crile::Engine) {
        self.check_scene_texture(&mut engine.gfx);

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

        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0.))
            .show(&ctx, |ui| {
                egui_dock::DockArea::new(&mut self.dock_state).show_inside(ui, &mut self.state);
            });

        self.egui.end_frame(engine, ctx);
    }

    fn render(&mut self, engine: &mut crile::Engine) {
        if let Some(texture) = &self.texture_output {
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
        if event == &crile::Event::WindowClose {
            engine.request_close();
        }

        self.egui.event(engine, event);
    }
}

impl CrileEditorApp {
    pub fn check_scene_texture(&mut self, gfx: &mut crile::GraphicsContext) {
        if self.state.viewport_size.x == 0. || self.state.viewport_size.y == 0. {
            return;
        }

        // If the viewport size is different from the texture output
        let resized = match self.texture_output {
            None => true,
            Some(ref texture) => texture.view().size().as_vec2() != self.state.viewport_size,
        };

        if resized {
            if let Some(texture) = self.texture_output.take() {
                self.egui.unregister_texture(&texture);
            }

            let texture = crile::Texture::new_render_attach(
                &gfx.wgpu,
                self.state.viewport_size.x as u32,
                self.state.viewport_size.y as u32,
            )
            .into();

            self.state.texture_id = Some(self.egui.register_texture(&texture));
            self.texture_output = Some(texture);
            self.state.scene.set_viewport(self.state.viewport_size);
        }
    }
}

fn main() {
    crile::run_app::<CrileEditorApp>().unwrap();
}
