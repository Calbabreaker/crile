use crile::Scene;

use crate::{project::Project, sections::WindowKind, EditorState};

pub fn show(state: &mut EditorState, ui: &mut egui::Ui) {
    egui::menu::bar(ui, |ui| {
        ui.menu_button("File", |ui| {
            ui.set_width(200.);
            if crile_egui::button_shorcut(ui, "Open Project...", "Ctrl+O").clicked() {
                open_project(state);
                ui.close_menu();
            }

            if crile_egui::button_shorcut(ui, "New Scene", "Ctrl+N").clicked() {
                new_scene(state);
                ui.close_menu();
            }

            if crile_egui::button_shorcut(ui, "Save Scene", "Ctrl+S").clicked() {
                save_scene(state);
                ui.close_menu();
            }

            if crile_egui::button_shorcut(ui, "Save Scene As...", "Ctrl+Shift+S").clicked() {
                save_scene_as(state);
                ui.close_menu();
            }

            if crile_egui::button_shorcut(ui, "Load Scene", "Ctrl+L").clicked() {
                load_scene(state);
                ui.close_menu();
            }
        });

        ui.menu_button("Edit", |ui| {
            if ui.button("Preferences...").clicked() {
                open_preferences(state);
                ui.close_menu();
            }
        })

        // ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        //     egui::Frame::default()
        //         .fill(egui::Color32::from_white_alpha(4))
        //         .inner_margin(4.)
        //         .rounding(4.)
        //         .show(ui, |ui| {
        //             if ui.small_button("▶").clicked() {
        //                 log::info!("Playing");
        //             }
        //         })
        // });
    });

    ui.input(|input| {
        if input.modifiers.command {
            if input.key_pressed(egui::Key::S) {
                if input.modifiers.shift {
                    save_scene_as(state);
                } else {
                    save_scene(state);
                }
            } else if input.key_pressed(egui::Key::O) {
                open_project(state);
            } else if input.key_pressed(egui::Key::L) {
                load_scene(state);
            } else if input.key_pressed(egui::Key::N) {
                new_scene(state);
            }
        }
    })
}

fn open_project(state: &mut EditorState) {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter("Crile project", &["crile"])
        .set_file_name("project.crile")
        .set_directory(std::env::current_dir().unwrap_or_default())
        .pick_file()
    {
        if let Some(project) = Project::load(path) {
            state.project = project;
        }

        if let Some(main_scene) = &state.project.main_scene {
            state.load_scene(Some(main_scene.clone()));
        }
    }
}

fn new_scene(state: &mut EditorState) {
    state.scene = Scene::default();
    state.active_scene_path = None;
}

fn save_scene(state: &mut EditorState) {
    state.save_scene(state.active_scene_path.clone());
}

fn save_scene_as(state: &mut EditorState) {
    state.save_scene(None);
}

fn load_scene(state: &mut EditorState) {
    state.load_scene(None);
    state.scene.set_viewport(state.viewport_size);
}

fn open_preferences(state: &mut EditorState) {
    state.window_open = WindowKind::Preferences;
}
