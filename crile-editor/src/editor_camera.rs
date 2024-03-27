#[derive(Default)]
pub struct EditorCamera2D {
    position: glam::Vec2,
    camera: crile::CameraComponent,
}

impl EditorCamera2D {
    pub fn view_projection(&self) -> glam::Mat4 {
        self.camera.projection() * glam::Mat4::from_translation(self.position.extend(0.))
    }

    pub fn process_input(&mut self, input: &egui::InputState) {
        dbg!(input.pointer.primary_down());
        if input.pointer.primary_down() {
            let speed = 1.0 / self.camera.orthographic_zoom;
            self.position += crile_egui::from_egui_vec(input.pointer.delta()) * speed;
        }

        self.camera.orthographic_zoom = f32::clamp(
            self.camera.orthographic_zoom + input.smooth_scroll_delta.y / 20.,
            0.01,
            100.0,
        );
    }

    pub fn set_viewport(&mut self, viewport_size: glam::Vec2) {
        self.camera.viewport_size = viewport_size;
    }
}
