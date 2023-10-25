#[derive(Default)]
pub struct SceneHierachyPanel {}

impl SceneHierachyPanel {
    pub fn show_scene(&mut self, ctx: &egui::Context, scene: &mut crile::Scene) {
        egui::SidePanel::left("Scene Hierachy").show(ctx, |ui| {
            let mut iter = scene.world.iter();
            while let Some(entity) = iter.next_entity() {
                if let Some(ident) = entity.get::<crile::IdentifierComponent>() {
                    let response = egui::CollapsingHeader::new(&ident.name)
                        .id_source(entity.id())
                        .selectable(true)
                        // .selected(selected)
                        // .open(open)
                        .show(ui, |ui| {});
                }
            }
        });
    }
}
