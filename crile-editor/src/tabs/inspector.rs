use crile_egui::EguiInspectable;

use crate::tabs::{EditorState, Selection};

pub fn ui(state: &mut EditorState, ui: &mut egui::Ui) {
    if let Selection::Entity(id) = state.selection {
        let entity = state.scene.world.entity(id);
        let meta = entity.get::<crile::MetaDataComponent>().unwrap();
        ui.text_edit_singleline(&mut meta.name);

        component_ui::<crile::TransformComponent>(ui, &entity);
        component_ui::<crile::SpriteRendererComponent>(ui, &entity);
        component_ui::<crile::CameraComponent>(ui, &entity);
    }
}

fn component_ui<T: EguiInspectable + 'static>(ui: &mut egui::Ui, entity: &crile::EntityRef) {
    if let Some(component) = entity.get::<T>() {
        component.ui(ui);
    }
}
