#[derive(Default)]
pub struct InspectorPanel {}

impl InspectorPanel {
    pub fn show_entity(&mut self, ctx: &egui::Context, entity: crile::EntityRef) {
        egui::SidePanel::right("Inspector").show(ctx, |ui| {
            let meta = entity.get::<crile::MetaDataComponent>().unwrap();
            ui.text_edit_singleline(&mut meta.name);
        });
    }
}
