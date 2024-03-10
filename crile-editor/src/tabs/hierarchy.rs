use crate::{EditorState, Selection};

pub fn ui(state: &mut EditorState, ui: &mut egui::Ui) {
    ui.add_space(5.);
    let mut action = HierachyAction::None;

    let root_entity = state.scene.root_entity();
    let root_meta = root_entity.get::<crile::MetaDataComponent>().unwrap();

    for child_id in &root_meta.children {
        display_entity(
            ui,
            &mut state.selection,
            *child_id,
            &state.scene.world,
            &mut action,
        );
    }

    ui.interact(
        egui::Rect::from_min_size(ui.cursor().left_top(), ui.available_size()),
        ui.make_persistent_id("hierachy context"),
        egui::Sense::click(),
    )
    .context_menu(|ui| {
        if ui.button("Add entity").clicked() {
            action = HierachyAction::AddChildEntity(None);
            ui.close_menu();
        }
    });

    match action {
        HierachyAction::AddChildEntity(parent_id) => {
            state.scene.spawn(
                (
                    crile::MetaDataComponent {
                        name: "Empty entity".to_owned(),
                        ..Default::default()
                    },
                    crile::TransformComponent::default(),
                ),
                parent_id,
            );
        }
        HierachyAction::DestroyEntity(id) => {
            state.scene.world.despawn(id);
        }
        HierachyAction::None => (),
    }
}

pub enum HierachyAction {
    None,
    AddChildEntity(Option<crile::EntityId>),
    DestroyEntity(crile::EntityId),
}

fn display_entity(
    ui: &mut egui::Ui,
    selection: &mut Selection,
    id: crile::EntityId,
    world: &crile::World,
    action: &mut HierachyAction,
) {
    let entity = world.entity(id);
    let meta = entity.get::<crile::MetaDataComponent>().unwrap();

    let has_children = !meta.children.is_empty();
    let header_state = egui::collapsing_header::CollapsingState::load_with_default_open(
        ui.ctx(),
        ui.make_persistent_id(id),
        true,
    );

    let mut show_header = |ui: &mut egui::Ui| {
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
            let response = ui.selectable_label(
                *selection == Selection::Entity(id),
                egui::RichText::new(&meta.name),
            );
            // let response = ui.add_sized(egui::vec2(ui.available_width(), 0.), label);
            if response.clicked() {
                *selection = Selection::Entity(id)
            }

            response.context_menu(|ui| {
                if ui.button("Add entity").clicked() {
                    *action = HierachyAction::AddChildEntity(Some(id));
                    ui.close_menu();
                }

                if ui.button("Destroy").clicked() {
                    *action = HierachyAction::DestroyEntity(id);
                    ui.close_menu();
                }
            });
        });
    };

    if has_children {
        header_state
            .show_header(ui, show_header)
            .body_unindented(|ui| {
                ui.indent(id, |ui| {
                    for child_id in &meta.children {
                        display_entity(ui, selection, *child_id, world, action);
                    }
                });
            });
    } else {
        show_header(ui);
    }
}
