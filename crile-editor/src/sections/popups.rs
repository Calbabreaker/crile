use crate::{EditorState, PopupKind};

pub fn show(ctx: &egui::Context, state: &mut EditorState, engine: &crile::Engine) {
    let mut open = true;
    let popup = egui::Window::new("Popup")
        .default_pos(ctx.screen_rect().size().to_pos2() / 2.)
        .open(&mut open)
        .resizable(false);

    match state.popup_open {
        PopupKind::Preferences => {
            popup.show(ctx, |ui| {
                egui::Resize::default().with_stroke(false).show(ui, |ui| {
                    state.preferences.show(ui);
                });
            });
        }
        PopupKind::Stats => {
            popup.show(ctx, |ui| {
                ui.label(format!("FPS: {}", engine.time.framerate()));
            });
        }
        PopupKind::None => (),
    }

    if !open {
        state.popup_open = PopupKind::None;
    }
}
