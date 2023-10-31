use crate::inspector_panel::InspectorPanel;

#[derive(Default)]
pub struct SceneHierachyPanel {
    selected_entity_id: crile::EntityId,
}

impl SceneHierachyPanel {
    pub fn show_scene(
        &mut self,
        ctx: &egui::Context,
        scene: &mut crile::Scene,
        inspector_panel: &mut InspectorPanel,
    ) {
        egui::SidePanel::left("Hierachy").show(ctx, |ui| {
            for (id, (meta,)) in scene.world.query::<(crile::MetaDataComponent,)>() {
                #[allow(deprecated)]
                let response = egui::CollapsingHeader::new(&meta.name)
                    .id_source(id)
                    .selectable(true)
                    .selected(self.selected_entity_id == id)
                    // .open(open)
                    .show(ui, |_| {});

                if response.header_response.clicked() {
                    self.selected_entity_id = id;
                }
            }
        });

        inspector_panel.show_entity(ctx, scene.world.entity(self.selected_entity_id));
    }
}
