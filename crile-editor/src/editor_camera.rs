use crate::Preferences;

#[derive(Default)]
pub struct EditorCamera2D {
    position: glam::Vec2,
    camera: crile::CameraComponent,
}

impl EditorCamera2D {
    pub fn view_projection(&self) -> glam::Mat4 {
        self.camera.projection() * glam::Mat4::from_translation(self.position.extend(0.))
    }

    pub fn process_input(&mut self, input: &egui::InputState, preferences: &Preferences) {
        if input.pointer.secondary_down() {
            let speed = 1.0 / self.camera.orthographic_zoom;
            self.position += crile_egui::from_egui_vec(input.pointer.delta()) * speed;
        }

        // Change zoom amount based on zoom so zooming close would feel the same
        let zoom_speed = preferences.zoom_speed * 0.1;
        let zoom_amount = input.smooth_scroll_delta.y * zoom_speed * self.camera.orthographic_zoom;
        self.camera.orthographic_zoom =
            f32::clamp(self.camera.orthographic_zoom + zoom_amount, 0.001, 100.0);
    }

    pub fn set_viewport(&mut self, viewport_size: glam::Vec2) {
        self.camera.viewport_size = viewport_size;
    }
}
