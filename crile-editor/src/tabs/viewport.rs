use crate::EditorState;

pub fn show(state: &mut EditorState, ui: &mut egui::Ui) {
    state.viewport_size = glam::vec2(ui.available_width(), ui.available_height());

    if let Some(id) = state.viewport_texture_id {
        ui.image(egui::ImageSource::Texture(egui::load::SizedTexture::new(
            id,
            ui.available_size(),
        )));
    }
}

pub fn check_texture(
    state: &mut EditorState,
    wgpu: &crile::WGPUContext,
    egui: &mut crile_egui::EguiContext,
) {
    if state.viewport_size.x == 0. || state.viewport_size.y == 0. {
        return;
    }

    // If the viewport size is different from the texture output
    let resized = match state.viewport_texture {
        None => true,
        Some(ref texture) => texture.view().size().as_vec2() != state.viewport_size,
    };

    if resized {
        if let Some(texture) = state.viewport_texture.take() {
            egui.unregister_texture(&texture);
        }

        let texture = crile::Texture::new_render_attach(
            wgpu,
            state.viewport_size.x as u32,
            state.viewport_size.y as u32,
        )
        .into();

        state.viewport_texture_id = Some(egui.register_texture(&texture));
        state.viewport_texture = Some(texture);
        state.scene.set_viewport(state.viewport_size);
    }
}
