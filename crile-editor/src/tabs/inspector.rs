use crile_egui::EguiInspectable;

use crate::{EditorState, Selection};

pub fn ui(state: &mut EditorState, ui: &mut egui::Ui) {
    if let Selection::Entity(id) = state.selection {
        let mut entity = state.scene.world.entity(id);
        let meta = entity.get::<crile::MetaDataComponent>().unwrap();
        ui.add_sized(
            egui::vec2(ui.available_width(), 0.),
            egui::TextEdit::singleline(&mut meta.name),
        );

        macro_rules! inspect_components {
            ( [$($component: ty),*]) => {{
                $( inspect_component::<$component>(ui, &entity); )*
            }};
        }

        crile::with_components!(inspect_components);

        ui.separator();

        ui.vertical_centered(|ui| {
            let response = ui.add_sized(
                egui::vec2(ui.available_width() / 2., 24.),
                egui::Button::new("Add component"),
            );

            let popup_id = ui.make_persistent_id("add component");
            if response.clicked() {
                ui.memory_mut(|mem| mem.toggle_popup(popup_id));
            }

            egui::popup_below_widget(ui, popup_id, &response, |ui| {
                macro_rules! add_component_buttons {
                    ( [$($component: ty),*]) => {{
                        $( add_component_button::<$component>(ui, &mut entity); )*
                    }};
                }

                crile::with_components!(add_component_buttons);
            });
        });
    }
}

fn inspect_component<T: EguiInspectable + 'static>(ui: &mut egui::Ui, entity: &crile::EntityRef) {
    if let Some(component) = entity.get::<T>() {
        egui::CollapsingHeader::new(T::pretty_name())
            .default_open(true)
            .show_background(true)
            .show_unindented(ui, |ui| {
                component.inspect(ui);
            });
    }
}

fn add_component_button<T: EguiInspectable + Default + 'static>(
    ui: &mut egui::Ui,
    entity: &mut crile::EntityRef,
) {
    if !entity.has::<T>() && ui.button(T::pretty_name()).clicked() {
        entity.add(T::default())
    }
}
