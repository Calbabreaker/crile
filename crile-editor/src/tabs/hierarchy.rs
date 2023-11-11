use crate::tabs::{EditorState, Selection};

pub fn ui(state: &mut EditorState, ui: &mut egui::Ui) {
    for (id, (meta,)) in state.scene.world.query::<(crile::MetaDataComponent,)>() {
        #[allow(deprecated)]
        let response = egui::CollapsingHeader::new(&meta.name)
            .id_source(id)
            .selectable(true)
            .selected(state.selection == Selection::Entity(id))
            // .open(open)
            .show(ui, |_| {});

        if response.header_response.clicked() {
            state.selection = Selection::Entity(id)
        }
    }
}
