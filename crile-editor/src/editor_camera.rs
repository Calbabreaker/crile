use crate::preferences::Preferences;

#[derive(Default)]
pub struct EditorCamera2D {
    position: glam::Vec2,
    pub camera: crile::CameraComponent,
}

impl EditorCamera2D {
    pub fn update(&mut self) {
        self.camera
            .update_projection(glam::Mat4::from_translation(self.position.extend(0.)));
    }

    pub fn view_projection(&self) -> glam::Mat4 {
        self.camera.view_projection
    }

    pub fn process_input(
        &mut self,
        input: &egui::InputState,
        viewport_offset: egui::Pos2,
        preferences: &Preferences,
    ) {
        // The mouse position offseted by the viewport position
        let mouse_position = input
            .pointer
            .latest_pos()
            .map(|pos| glam::Vec2::new(pos.x - viewport_offset.x, pos.y - viewport_offset.y))
            .unwrap_or_default();

        let camera_zoom = self.camera.orthographic_zoom;
        if input.pointer.secondary_down() {
            self.position -= crile_egui::from_egui_vec(input.pointer.delta()) / camera_zoom;
        }

        // Change zoom amount based on zoom so zooming close would feel the same
        let zoom_speed = preferences.zoom_speed * 0.1;
        let zoom_amount = input.smooth_scroll_delta.y * zoom_speed * camera_zoom;
        self.camera.orthographic_zoom = f32::clamp(camera_zoom + zoom_amount, 0.001, 100.0);

        if zoom_amount != 0. {
            // Calculate where to move the camera so it will feel like we zoomed into where the cursor is pointing at
            let cursor_world_position = self.camera.screen_to_world(mouse_position);
            let offset = self.position - cursor_world_position;

            // Scale the offset according to the zoom amount
            // TODO: this does not feel right
            let scaled_offset =
                offset * (camera_zoom - self.camera.orthographic_zoom) / camera_zoom;
            self.position += scaled_offset;
        }

        self.camera.dirty = true;
    }

    pub fn set_viewport(&mut self, viewport_size: glam::Vec2) {
        self.camera.viewport_size = viewport_size;
    }
}
