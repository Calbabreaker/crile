use crile_egui::EguiInspectable;

use crate::{EditorState, Selection};

pub fn ui(state: &mut EditorState, ui: &mut egui::Ui) {
    if let Selection::Entity(id) = state.selection {
        let entity = state.scene.world.entity(id);
        let meta = entity.get::<crile::MetaDataComponent>().unwrap();
        ui.text_edit_singleline(&mut meta.name);

        egui::ScrollArea::vertical().show(ui, |ui| {
            macro_rules! inspect_components {
                ( [$($component: ty),*]) => {{
                    $( inspect_component::<$component>(ui, &entity); )*
                }};
            }

            crile::with_components!(inspect_components);
        });
    }
}

fn inspect_component<T: EguiInspectable + 'static>(ui: &mut egui::Ui, entity: &crile::EntityRef) {
    if let Some(component) = entity.get::<T>() {
        ui.label(T::pretty_name());
        component.inspect(ui);
    }
}
