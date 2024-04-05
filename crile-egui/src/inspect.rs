pub fn inspect_f32(ui: &mut egui::Ui, value: &mut f32) {
    ui.vertical_centered_justified(|ui| {
        ui.add(egui::DragValue::new(value).speed(0.01));
    });
}

pub fn inspect_vec3(ui: &mut egui::Ui, value: &mut glam::Vec3) {
    ui.horizontal(|ui| {
        ui.spacing_mut().interact_size.x = 0.;
        // Make sure vec3 control resizes to fit
        let font_size = ui.style().text_styles[&egui::TextStyle::Body].size;
        let size = egui::vec2(
            ui.available_width() / 3. - font_size - ui.spacing().item_spacing.x,
            ui.spacing().interact_size.y,
        )
        .floor();

        ui.label("X");
        ui.add_sized(size, egui::DragValue::new(&mut value.x).speed(0.01));
        ui.label("Y");
        ui.add_sized(size, egui::DragValue::new(&mut value.y).speed(0.01));
        ui.label("Z");
        ui.add_sized(size, egui::DragValue::new(&mut value.z).speed(0.01));
    });
}

pub fn inspect_color(ui: &mut egui::Ui, value: &mut crile::Color) {
    ui.color_edit_button_rgba_premultiplied(&mut value.0);
}

pub fn inspect_asset_path(ui: &mut egui::Ui, asset_path: &mut crile::AssetPath) {
    let text = asset_path
        .path
        .as_ref()
        .and_then(|path| path.to_str())
        .unwrap_or("Choose file");

    if ui
        .add_sized(ui.available_size(), egui::Button::new(text))
        .clicked()
    {
        asset_path.open_picker = true;
    }
}
