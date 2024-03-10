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
        inspect_with_label(ui, "Translation", &mut self.translation);
        ui.end_row();

        inspect_with_label(ui, "Rotation", &mut self.rotation);
        ui.end_row();

        inspect_with_label(ui, "Scale", &mut self.scale);
        ui.end_row();
    }
}

impl EguiInspectable for crile::SpriteRendererComponent {
    fn pretty_name() -> &'static str {
        "Sprite Component"
    }

    fn inspect(&mut self, ui: &mut egui::Ui) {
        inspect_with_label(ui, "Color", &mut self.color);
        ui.end_row();
    }
}

impl EguiInspectable for crile::CameraComponent {
    fn pretty_name() -> &'static str {
        "Camera Component"
    }

    fn inspect(&mut self, ui: &mut egui::Ui) {
        inspect_with_label(ui, "Near", &mut self.near);
        ui.end_row();
        inspect_with_label(ui, "Far", &mut self.far);
        ui.end_row();
        inspect_with_label(ui, "Orthographic Size", &mut self.ortho_size);
        ui.end_row();
    }
}

fn inspect_with_label(ui: &mut egui::Ui, label: &str, inpectable: &mut impl EguiInspectable) {
    ui.label(label);
    inpectable.inspect(ui);
}

impl EguiInspectable for f32 {
    fn inspect(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered_justified(|ui| {
            ui.add(egui::DragValue::new(self).speed(0.01));
        });
    }
}

impl EguiInspectable for glam::Vec3 {
    fn inspect(&mut self, ui: &mut egui::Ui) {
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
        let mut array = self.to_array();
        ui.color_edit_button_rgba_premultiplied(&mut array);
        *self = Self::new(array[0], array[1], array[2], array[3])
    }
}
