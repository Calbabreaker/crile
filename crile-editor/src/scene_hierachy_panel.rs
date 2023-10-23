#[derive(Default)]
pub struct SceneHierachyPanel {}

impl SceneHierachyPanel {
    pub fn show_scene(&mut self, ctx: &egui::Context, scene: &mut crile::Scene) {
        egui::SidePanel::left("Scene Hierachy").show(ctx, |ui| {
            for entity in scene.world.iter() {
                if let Some(id) = entity.get::<crile::IdentifierComponent>() {
                    dbg!(&id);
                    ui.label(&id.name);
                }
            }
        });
    }
}
