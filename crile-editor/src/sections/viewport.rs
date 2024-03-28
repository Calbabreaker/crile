use crate::{sections::SceneState, EditorState};

pub fn show(state: &mut EditorState, ui: &mut egui::Ui) {
    state.viewport_size = glam::uvec2(ui.available_width() as u32, ui.available_height() as u32);
    if let Some(id) = state.viewport_texture_id {
        let response = ui
            .image(egui::ImageSource::Texture(egui::load::SizedTexture::new(
                id,
                ui.available_size(),
            )))
            .interact(egui::Sense::click_and_drag());

        if state.scene_state == SceneState::Editing && response.hovered() {
            ui.input(|i| state.editor_camera.process_input(i));
        }
    }
}

pub fn check_texture(
    state: &mut EditorState,
    wgpu: &crile::WGPUContext,
    egui: &mut crile_egui::EguiContext,
) {
    if state.viewport_size.x == 0
        || state.viewport_size.y == 0
        || state.viewport_size.x >= wgpu.limits.max_texture_dimension_2d
        || state.viewport_size.y >= wgpu.limits.max_texture_dimension_2d
    {
        return;
    }

    // If the viewport size is different from the texture output
    let resized = match state.viewport_texture {
        None => true,
        Some(ref texture) => texture.view().size() != state.viewport_size,
    };

    if resized {
        if let Some(texture) = state.viewport_texture.take() {
            egui.unregister_texture(&texture);
        }

        let texture = crile::Texture::new_render_attach(wgpu, state.viewport_size).into();

        state.viewport_texture_id = Some(egui.register_texture(&texture));
        state.viewport_texture = Some(texture);
        state.scene.set_viewport(state.viewport_size);

        state.depth_texture = Some(crile::Texture::new_depth(wgpu, state.viewport_size));
        state
            .editor_camera
            .set_viewport(state.viewport_size.as_vec2());
    }
}
