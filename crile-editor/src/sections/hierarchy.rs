use crate::{EditorState, Selection};

pub enum HierachyAction {
    None,
    AddChildEntity(usize),
    DestroyEntity(usize),
}

pub fn show(ui: &mut egui::Ui, state: &mut EditorState) {
    ui.add_space(5.);
    let mut action = HierachyAction::None;

    display_entity(
        ui,
        &mut state.selection,
        crile::Scene::ROOT_INDEX,
        &state.active_scene,
        &mut action,
    );

    ui.interact(
        egui::Rect::from_min_size(ui.cursor().left_top(), ui.available_size()),
        ui.make_persistent_id("hierachy context"),
        egui::Sense::click(),
    )
    .context_menu(|ui| {
        if ui.button("Add entity").clicked() {
            action = HierachyAction::AddChildEntity(0);
            ui.close_menu();
        }
    });

    match action {
        HierachyAction::AddChildEntity(parent_index) => {
            state.active_scene.spawn(
                "Empty",
                (crile::TransformComponent::default(),),
                parent_index,
            );
        }
        HierachyAction::DestroyEntity(index) => {
            state.active_scene.despawn(index);
        }
        HierachyAction::None => (),
    }
}

fn display_entity(
    ui: &mut egui::Ui,
    selection: &mut Selection,
    index: usize,
    scene: &crile::Scene,
    action: &mut HierachyAction,
) {
    let node = scene.get_node(index).unwrap();

    let header_state = egui::collapsing_header::CollapsingState::load_with_default_open(
        ui.ctx(),
        ui.make_persistent_id(index),
        true,
    );

    let mut show_header = |ui: &mut egui::Ui| {
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
            let response = ui.selectable_label(*selection == Selection::Entity(index), &node.name);
            if response.clicked() {
                *selection = Selection::Entity(index)
            }

            response.context_menu(|ui| {
                if ui.button("Add entity").clicked() {
                    *action = HierachyAction::AddChildEntity(index);
                    ui.close_menu();
                }

                if index != crile::Scene::ROOT_INDEX && ui.button("Destroy").clicked() {
                    *action = HierachyAction::DestroyEntity(index);
                    ui.close_menu();
                }
            });
        });
    };

    if !node.children.is_empty() {
        header_state
            .show_header(ui, show_header)
            .body_unindented(|ui| {
                ui.indent(index, |ui| {
                    for id in node.children.iter() {
                        let index = scene.id_to_index(*id);
                        display_entity(ui, selection, index, scene, action);
                    }
                });
            });
    } else {
        show_header(ui);
    }
}
