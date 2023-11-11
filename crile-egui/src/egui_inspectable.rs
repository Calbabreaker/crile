pub trait EguiInspectable {
    fn ui(&mut self, ui: &mut egui::Ui);
}

impl EguiInspectable for f32 {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::DragValue::new(self).speed(0.01));
    }
}

impl EguiInspectable for glam::Vec3 {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            inspect_with_label(ui, "X", &mut self.x);
            inspect_with_label(ui, "Y", &mut self.y);
            inspect_with_label(ui, "Z", &mut self.z);
        });
    }
}

impl EguiInspectable for crile::TransformComponent {
    fn ui(&mut self, ui: &mut egui::Ui) {
        label_grid("Transform Component", ui, |ui| {
            inspect_with_label(ui, "Translation", &mut self.translation);
            ui.end_row();

            inspect_with_label(ui, "Rotation", &mut self.rotation);
            ui.end_row();

            inspect_with_label(ui, "Scale", &mut self.scale);
            ui.end_row();
        });
    }
}

impl EguiInspectable for crile::Color {
    fn ui(&mut self, ui: &mut egui::Ui) {
        let mut rgba = egui::Color32::from_rgba_premultiplied(
            (self.r * 255.) as u8,
            (self.g * 255.) as u8,
            (self.b * 255.) as u8,
            (self.a * 255.) as u8,
        );
        ui.color_edit_button_srgba(&mut rgba);
        // *self = Self::from_rgba(rgba.r(), rgba.g(), rgba.b(), rgba.a());
    }
}

impl EguiInspectable for crile::SpriteRendererComponent {
    fn ui(&mut self, ui: &mut egui::Ui) {
        label_grid("Sprite Renderer", ui, |ui| {
            inspect_with_label(ui, "Color", &mut self.color);
            ui.end_row();
        });
    }
}

impl EguiInspectable for crile::CameraComponent {
    fn ui(&mut self, ui: &mut egui::Ui) {
        label_grid("Camera", ui, |ui| {
            inspect_with_label(ui, "Near", &mut self.near);
            ui.end_row();
            inspect_with_label(ui, "Far", &mut self.far);
            ui.end_row();
            inspect_with_label(ui, "Orthographic Size", &mut self.ortho_size);
            ui.end_row();
        });
    }
}

fn label_grid(name: &str, ui: &mut egui::Ui, func: impl FnOnce(&mut egui::Ui)) {
    ui.label(name);
    egui::Grid::new(name)
        .num_columns(2)
        .spacing([40.0, 4.0])
        .show(ui, func);
}

fn inspect_with_label(ui: &mut egui::Ui, label: &str, inpectable: &mut impl EguiInspectable) {
    ui.label(label);
    inpectable.ui(ui);
}
