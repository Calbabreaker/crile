use crate::tabs::{EditorState, Selection};

pub fn show(state: &mut EditorState, ui: &mut egui::Ui) {
    if let Selection::Entity(id) = state.selection {
        let entity = state.scene.world.entity(id);
        let meta = entity.get::<crile::MetaDataComponent>().unwrap();
        ui.text_edit_singleline(&mut meta.name);

        egui::Grid::new("my_grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                if let Some(transform) = entity.get::<crile::TransformComponent>() {
                    ui.label("Translation: ");
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(&mut transform.translation.x).speed(0.1));
                        ui.add(egui::DragValue::new(&mut transform.translation.y).speed(0.1));
                        ui.add(egui::DragValue::new(&mut transform.translation.z).speed(0.1));
                    });
                    ui.end_row();
                }
            });
    }
}
