pub struct Preferences {
    pub ui_scale: f32,
}

impl Default for Preferences {
    fn default() -> Self {
        Self { ui_scale: 1. }
    }
}

impl Preferences {
    /// Returns whether or not apply has been clicked
    pub fn show(&mut self, ui: &mut egui::Ui) -> bool {
        egui::Grid::new("Options grid")
            .num_columns(2)
            .striped(true)
            .spacing([30.0, 4.0])
            .show(ui, |ui| {
                ui.label("Ui scale");
                ui.add(egui::Slider::new(&mut self.ui_scale, 0.5..=2.0).step_by(0.01));
                ui.end_row();
            });

        ui.add_space(5.);
        if ui.button("Apply").clicked() {
            return true;
        }

        false
    }
}
