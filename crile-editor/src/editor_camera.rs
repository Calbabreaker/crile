#[derive(Default)]
pub struct EditorCamera2D {
    position: glam::Vec2,
    camera: crile::CameraComponent,
}

impl EditorCamera2D {
    const ZOOM_SPEED: f32 = 0.05;

    pub fn view_projection(&self) -> glam::Mat4 {
        self.camera.projection() * glam::Mat4::from_translation(self.position.extend(0.))
    }

    pub fn process_input(&mut self, input: &egui::InputState) {
        if input.pointer.secondary_down() {
            let speed = 1.0 / self.camera.orthographic_zoom;
            self.position += crile_egui::from_egui_vec(input.pointer.delta()) * speed;
        }

        let zoom_amount =
            input.smooth_scroll_delta.y * Self::ZOOM_SPEED * self.camera.orthographic_zoom;
        self.camera.orthographic_zoom =
            f32::clamp(self.camera.orthographic_zoom + zoom_amount, 0.01, 100.0);
    }

    pub fn set_viewport(&mut self, viewport_size: glam::Vec2) {
        self.camera.viewport_size = viewport_size;
    }
}
