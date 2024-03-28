use crile::Scene;

use crate::{
    project::Project,
    sections::{SceneState, WindowKind},
    EditorState,
};

pub fn show(state: &mut EditorState, ui: &mut egui::Ui) {
    egui::menu::bar(ui, |ui| {
        ui.columns(3, |ui| {
            ui[0].horizontal(|ui| {
                left_menus(state, ui);
            });

            ui[1].vertical_centered(|ui| {
                if state.scene_state == SceneState::Running {
                    if ui.button("⏹").clicked() {
                        state.stop_scene();
                    }
                } else if ui.button("▶").clicked() {
                    state.play_scene();
                }
            });
        });

        ui.input(|input| {
            if input.modifiers.command {
                if input.key_pressed(egui::Key::S) {
                    if input.modifiers.shift {
                        state.save_scene(None);
                    } else {
                        state.save_scene(state.editor_scene_path.clone());
                    }
                } else if input.key_pressed(egui::Key::O) {
                    open_project(state);
                } else if input.key_pressed(egui::Key::L) {
                    state.load_scene(None);
                } else if input.key_pressed(egui::Key::N) {
                    new_scene(state);
                }
            }
        })
    });
}

fn left_menus(state: &mut EditorState, ui: &mut egui::Ui) {
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
            state.save_scene(state.editor_scene_path.clone());
            ui.close_menu();
        }

        if crile_egui::button_shorcut(ui, "Save Scene As...", "Ctrl+Shift+S").clicked() {
            state.save_scene(None);
            ui.close_menu();
        }

        if crile_egui::button_shorcut(ui, "Load Scene", "Ctrl+L").clicked() {
            state.load_scene(None);
            ui.close_menu();
        }
    });

    ui.menu_button("Edit", |ui| {
        if ui.button("Preferences...").clicked() {
            state.window_open = WindowKind::Preferences;
            ui.close_menu();
        }
    });
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
    state.stop_scene();
    state.scene = Scene::default();
    state.editor_scene_path = None;
}
