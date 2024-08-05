use crate::{EditorState, PopupKind};

pub fn show(
    ctx: &egui::Context,
    egui_ctx: &mut crile_egui::EguiContext,
    state: &mut EditorState,
    engine: &mut crile::Engine,
) {
    let title = match state.popup_open {
        PopupKind::Stats => "Stats",
        PopupKind::Preferences(_) => "Preferences",
        PopupKind::None => return,
    };

    let mut open = true;
    let popup = egui::Window::new(title)
        .default_pos(ctx.screen_rect().size().to_pos2() / 2.)
        .open(&mut open)
        .resizable(false);

    match state.popup_open {
        PopupKind::Preferences(ref mut temp_prefs) => {
            popup.show(ctx, |ui| {
                if temp_prefs.show(ui) {
                    state.preferences = temp_prefs.clone();
                    egui_ctx.set_ui_scale(state.preferences.ui_scale, engine.main_window().size());
                    engine.gfx.wgpu.set_vsync(state.preferences.vsync);
                }
            });
        }
        PopupKind::Stats => {
            popup.show(ctx, |ui| {
                ui.label(format!("FPS: {}", engine.time.frame_rate()));
            });
        }
        PopupKind::None => unreachable!(),
    }

    if !open {
        state.popup_open = PopupKind::None;
    }
}
