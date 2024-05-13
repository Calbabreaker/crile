use crate::{EditorState, PopupKind, SceneState};

pub fn show(
    ui: &mut egui::Ui,
    state: &mut EditorState,
    engine: &mut crile::Engine,
    event_loop: &crile::ActiveEventLoop,
) {
    egui::menu::bar(ui, |ui| {
        ui.columns(3, |ui| {
            ui[0].horizontal(|ui| {
                left_menus(state, ui);
            });

            ui[1].vertical_centered(|ui| {
                if matches!(state.scene_state, SceneState::Edting) {
                    if ui.button("▶").clicked() {
                        state.play_scene(engine, event_loop);
                    }
                } else if ui.button("⏹").clicked() {
                    state.stop_scene(engine);
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
                    state.open_project(None);
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
            state.open_project(None);
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

        if crile_egui::button_shorcut(ui, "Load Scene...", "Ctrl+L").clicked() {
            state.load_scene(None);
            ui.close_menu();
        }
    });

    ui.menu_button("Edit", |ui| {
        if ui.button("Preferences...").clicked() {
            state.popup_open = PopupKind::Preferences;
            ui.close_menu();
        }

        if ui.button("Stats...").clicked() {
            state.popup_open = PopupKind::Stats;
            ui.close_menu();
        }
    });
}

fn new_scene(state: &mut EditorState) {
    if matches!(state.scene_state, SceneState::Running(_)) {
        return;
    }

    state.active_scene = crile::Scene::default();
    state.editor_scene_path = None;
}
