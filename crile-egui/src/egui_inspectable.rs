pub trait EguiInspectable {
    fn inspect(&mut self, ui: &mut egui::Ui);
    fn pretty_name() -> &'static str {
        ""
    }
}

impl EguiInspectable for crile::TransformComponent {
    fn pretty_name() -> &'static str {
        "Transform Component"
    }

    fn inspect(&mut self, ui: &mut egui::Ui) {
        grid(ui, |ui| {
            inspect_with_label(ui, "Translation", &mut self.translation);
            ui.end_row();

            inspect_with_label(ui, "Rotation", &mut self.rotation);
            ui.end_row();

            inspect_with_label(ui, "Scale", &mut self.scale);
            ui.end_row();
        });
    }
}

impl EguiInspectable for crile::SpriteRendererComponent {
    fn pretty_name() -> &'static str {
        "Sprite Component"
    }

    fn inspect(&mut self, ui: &mut egui::Ui) {
        grid(ui, |ui| {
            inspect_with_label(ui, "Color", &mut self.color);
            ui.end_row();
        });
    }
}

impl EguiInspectable for crile::CameraComponent {
    fn pretty_name() -> &'static str {
        "Camera Component"
    }

    fn inspect(&mut self, ui: &mut egui::Ui) {
        grid(ui, |ui| {
            inspect_with_label(ui, "Near", &mut self.near);
            ui.end_row();
            inspect_with_label(ui, "Far", &mut self.far);
            ui.end_row();
            inspect_with_label(ui, "Orthographic Size", &mut self.ortho_size);
            ui.end_row();
        });
    }
}

fn grid(ui: &mut egui::Ui, func: impl FnOnce(&mut egui::Ui)) {
    egui::Grid::new(std::any::type_name_of_val(&func))
        .num_columns(2)
        .spacing([30.0, 4.0])
        .show(ui, func);
}

fn inspect_with_label(ui: &mut egui::Ui, label: &str, inpectable: &mut impl EguiInspectable) {
    ui.label(label);
    inpectable.inspect(ui);
}

impl EguiInspectable for f32 {
    fn inspect(&mut self, ui: &mut egui::Ui) {
        ui.add_sized(ui.available_size(), egui::DragValue::new(self).speed(0.01));
    }
}

impl EguiInspectable for glam::Vec3 {
    fn inspect(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.style_mut().spacing.interact_size.x = 0.;
            let font_size = ui.style().text_styles[&egui::TextStyle::Body].size;
            let item_spacing = ui.style().spacing.item_spacing;
            let size = egui::vec2(
                (ui.available_width() / 3.).floor() - font_size - item_spacing.x,
                ui.style().spacing.interact_size.y,
            );

            ui.label("X");
            ui.add_sized(size, egui::DragValue::new(&mut self.x).speed(0.01));
            ui.label("Y");
            ui.add_sized(size, egui::DragValue::new(&mut self.y).speed(0.01));
            ui.label("Z");
            ui.add_sized(size, egui::DragValue::new(&mut self.z).speed(0.01));
        });
    }
}

impl EguiInspectable for crile::Color {
    fn inspect(&mut self, ui: &mut egui::Ui) {
        let mut rgba = egui::Color32::from_rgba_premultiplied(
            (self.r * 255.) as u8,
            (self.g * 255.) as u8,
            (self.b * 255.) as u8,
            (self.a * 255.) as u8,
        );
        ui.color_edit_button_srgba(&mut rgba);
        *self = Self::from_rgba(rgba.r(), rgba.g(), rgba.b(), rgba.a());
    }
}
