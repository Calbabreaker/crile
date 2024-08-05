use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(default)]
pub struct Preferences {
    pub ui_scale: f32,
    pub zoom_speed: f32,
    pub last_opened_project: Option<PathBuf>,
    pub vsync: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            ui_scale: 1.,
            last_opened_project: None,
            zoom_speed: 2.,
            vsync: true,
        }
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
                ui.add(egui::Slider::new(&mut self.ui_scale, 0.1..=2.0).step_by(0.01));
                ui.end_row();

                ui.label("Zoom speed");
                ui.add(egui::Slider::new(&mut self.zoom_speed, 0.5..=4.));
                ui.end_row();

                ui.label("Vsync");
                ui.checkbox(&mut self.vsync, "");
                ui.end_row();
            });

        ui.add_space(5.);
        if ui.button("Apply").clicked() {
            self.save();
            true
        } else {
            false
        }
    }

    pub fn save(&self) -> bool {
        let data = toml::to_string(self).unwrap();
        crile::write_file(&Self::file_path().unwrap(), &data)
    }

    pub fn load() -> Option<Self> {
        let file_path = Self::file_path()?;
        let source = crile::read_file(&file_path)?;
        toml::from_str(&source)
            .inspect_err(|err| log::error!("Failed to load {file_path:?}: {err}"))
            .ok()?
    }

    fn file_path() -> Option<PathBuf> {
        crile::get_data_path().map(|path| path.join("preferences.toml"))
    }
}
