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
        egui::SidePanel::left("Hierachy")
            .resizable(true)
            .show(ctx, |ui| {
                let mut iter = scene.world.iter();
                while let Some(entity) = iter.next_entity() {
                    if let Some(meta) = entity.get::<crile::MetaDataComponent>() {
                        #[allow(deprecated)]
                        let response = egui::CollapsingHeader::new(&meta.name)
                            .id_source(entity.id())
                            .selectable(true)
                            .selected(self.selected_entity_id == entity.id())
                            // .open(open)
                            .show(ui, |ui| {});

                        if response.header_response.clicked() {
                            self.selected_entity_id = entity.id();
                        }
                    }
                }
            });

        inspector_panel.show_entity(ctx, scene.world.entity(self.selected_entity_id));
    }
}
