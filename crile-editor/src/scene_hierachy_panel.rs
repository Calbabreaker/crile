#[derive(Default)]
pub struct SceneHierachyPanel {}

impl SceneHierachyPanel {
    pub fn show_scene(&mut self, ctx: &egui::Context, scene: &mut crile::Scene) {
        egui::SidePanel::left("Scene Hierachy").show(ctx, |ui| {
            let mut iter = scene.world.iter();
            while let Some(entity) = iter.next_entity() {
                if let Some(id) = entity.get::<crile::IdentifierComponent>() {
                    ui.label(&id.name);
                }
            }
        });
    }
}
