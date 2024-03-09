use crate::{EditorState, Selection};

pub fn ui(state: &mut EditorState, ui: &mut egui::Ui) {
    for (id, (meta,)) in state.scene.world.query::<(crile::MetaDataComponent,)>() {
        let has_children = !meta.children.is_empty();
        let is_open = if !has_children { Some(false) } else { None };

        let res = ui.selectable_label(state.selection == Selection::Entity(id), &meta.name);
        if res.clicked() {
            state.selection = Selection::Entity(id)
        }

        // let egui_id = ui.make_persistent_id(id);
        // egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), egui_id, true)
        //     .show_header(ui, |ui| {
        //         let res = ui.selectable_label(state.selection == Selection::Entity(id), &meta.name);
        //         if res.clicked() {
        //             state.selection = Selection::Entity(id)
        //         }
        //     })
        //     .body(|ui| {});
    }
}
