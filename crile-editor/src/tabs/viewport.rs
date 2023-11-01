use crate::tabs::EditorState;

pub fn show(state: &mut EditorState, ui: &mut egui::Ui) {
    state.viewport_size = glam::vec2(ui.available_width(), ui.available_height());

    if let Some(id) = state.texture_id {
        ui.image(egui::ImageSource::Texture(egui::load::SizedTexture::new(
            id,
            ui.available_size(),
        )));
    }
}
