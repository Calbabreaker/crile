use crile_egui::EguiInspectable;

use crate::{EditorState, Selection};

pub fn ui(state: &mut EditorState, ui: &mut egui::Ui) {
    ui.add_space(5.);

    ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
        if let Selection::Entity(id) = state.selection {
            let mut entity = state.scene.world.entity_mut(id);
            let meta = entity.get::<crile::MetaDataComponent>().unwrap();
            ui.text_edit_singleline(&mut meta.name);
            ui.add_space(5.);

            macro_rules! inspect_components {
                ( [$($component: ty),*]) => {{
                    $( inspect_component::<$component>(ui, &mut entity); )*
                }};
            }

            crile::with_components!(inspect_components);

            ui.separator();

            ui.vertical_centered(|ui| {
                ui.menu_button("Add component", |ui| {
                    macro_rules! add_component_buttons {
                        ( [$($component: ty),*]) => {{
                            $( add_component_button::<$component>(ui, &mut entity); )*
                        }};
                    }

                    crile::with_components!(add_component_buttons);
                });
            });
        }
    });
}

fn inspect_component<T: EguiInspectable + 'static>(
    ui: &mut egui::Ui,
    entity: &mut crile::EntityMut,
) {
    if let Some(component) = entity.get::<T>() {
        // TODO: figure out how to make header fill available width
        let response = egui::CollapsingHeader::new(T::pretty_name())
            .default_open(true)
            .show_background(true)
            .show_unindented(ui, |ui| {
                ui.visuals_mut().widgets.noninteractive.bg_stroke.width = 0.;
                ui.spacing_mut().indent = 8.;

                ui.indent(T::pretty_name(), |ui| {
                    egui::Grid::new(T::pretty_name())
                        .num_columns(2)
                        .spacing([30.0, 4.0])
                        .show(ui, |ui| component.inspect(ui));
                });
            });

        response.header_response.context_menu(move |ui| {
            if ui.button("Remove component").clicked() {
                entity.remove::<T>();
                ui.close_menu();
            }
        });
    }
}

fn add_component_button<T: EguiInspectable + Default + 'static>(
    ui: &mut egui::Ui,
    entity: &mut crile::EntityMut,
) {
    if !entity.has::<T>() && ui.button(T::pretty_name()).clicked() {
        entity.add(T::default());
        ui.close_menu();
    }
}
