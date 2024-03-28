#[derive(Default)]
pub struct SceneViewport {
    pub depth_texture: Option<crile::Texture>,
    pub texture_id: Option<egui::TextureId>,
    pub texture: Option<crile::RefId<crile::Texture>>,
    pub size: glam::UVec2,
}

impl SceneViewport {
    pub fn check_texture(&mut self, wgpu: &crile::WGPUContext, egui: &mut crile_egui::EguiContext) {
        if self.size.x == 0
            || self.size.y == 0
            || self.size.x >= wgpu.limits.max_texture_dimension_2d
            || self.size.y >= wgpu.limits.max_texture_dimension_2d
        {
            return;
        }

        // If the viewport size is different from the texture output
        let resized = match self.texture {
            None => true,
            Some(ref texture) => texture.view().size() != self.size,
        };

        if resized {
            if let Some(texture) = self.texture.take() {
                egui.unregister_texture(&texture);
            }

            let texture = crile::Texture::new_render_attach(wgpu, self.size).into();

            self.texture_id = Some(egui.register_texture(&texture));
            self.texture = Some(texture);

            self.depth_texture = Some(crile::Texture::new_depth(wgpu, self.size));
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<egui::Response> {
        self.size = glam::uvec2(ui.available_width() as u32, ui.available_height() as u32);
        self.texture_id.map(|id| {
            ui.image(egui::ImageSource::Texture(egui::load::SizedTexture::new(
                id,
                ui.available_size(),
            )))
            .interact(egui::Sense::click_and_drag())
        })
    }

    pub fn get_render_pass<'a>(
        &'a self,
        engine: &'a mut crile::Engine,
    ) -> Option<crile::RenderPass> {
        // Render to the viewport texture to be displayed in the viewport panel
        self.texture.as_ref().map(|texture| {
            crile::RenderPass::new(
                &mut engine.gfx,
                Some(crile::Color::BLACK),
                self.depth_texture.as_ref(),
                Some(texture.view()),
            )
        })
    }
}
