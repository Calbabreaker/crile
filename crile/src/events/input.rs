use core::hash::Hash;

use crate::{ButtonState, EventKind, KeyCode, KeyModifiers, MouseButton};

struct InputState<T> {
    pressed: hashbrown::HashSet<T>,
    just_pressed: hashbrown::HashSet<T>,
    just_released: hashbrown::HashSet<T>,
}

impl<T: Eq + Hash> Default for InputState<T> {
    fn default() -> Self {
        Self {
            pressed: Default::default(),
            just_pressed: Default::default(),
            just_released: Default::default(),
        }
    }
}

impl<T: Copy + Eq + Hash> InputState<T> {
    fn press(&mut self, code: T) {
        self.pressed.insert(code);
        self.just_pressed.insert(code);
    }

    fn release(&mut self, code: T) {
        self.pressed.remove(&code);
        self.just_released.insert(code);
    }

    fn clear(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }
}

#[derive(Default)]
pub struct Input {
    key_state: InputState<KeyCode>,
    mouse_state: InputState<MouseButton>,
    mouse_position: glam::Vec2,
    key_modifiers: KeyModifiers,
}

impl Input {
    pub fn key_pressed(&self, key_code: KeyCode) -> bool {
        self.key_state.pressed.contains(&key_code)
    }

    pub fn key_just_pressed(&self, key_code: KeyCode) -> bool {
        self.key_state.just_pressed.contains(&key_code)
    }

    pub fn key_just_released(&self, key_code: KeyCode) -> bool {
        self.key_state.just_released.contains(&key_code)
    }

    pub fn mouse_pressed(&self, mouse_code: MouseButton) -> bool {
        self.mouse_state.pressed.contains(&mouse_code)
    }

    pub fn mouse_just_pressed(&self, mouse_code: MouseButton) -> bool {
        self.mouse_state.just_pressed.contains(&mouse_code)
    }

    pub fn mouse_just_released(&self, mouse_code: MouseButton) -> bool {
        self.mouse_state.just_released.contains(&mouse_code)
    }

    pub fn mouse_position(&self) -> glam::Vec2 {
        self.mouse_position
    }

    /// Gets an input vector with the specified keycodes for the negative and positive XY.
    pub fn get_vector(
        &self,
        negative_x: KeyCode,
        negative_y: KeyCode,
        positive_x: KeyCode,
        positive_y: KeyCode,
    ) -> glam::Vec2 {
        glam::Vec2::new(
            self.key_pressed(positive_x) as u32 as f32 - self.key_pressed(negative_x) as u32 as f32,
            self.key_pressed(negative_y) as u32 as f32 - self.key_pressed(positive_y) as u32 as f32,
        )
        .normalize_or_zero()
    }

    /// Update an internal state with crile::EventKind
    pub fn process_event(&mut self, kind: &EventKind) {
        match kind {
            EventKind::KeyInput {
                keycode,
                state,
                repeat: false,
                ..
            } => match state {
                ButtonState::Pressed => self.key_state.press(*keycode),
                ButtonState::Released => self.key_state.release(*keycode),
            },
            EventKind::MouseInput { button, state } => match state {
                ButtonState::Pressed => self.mouse_state.press(*button),
                ButtonState::Released => self.mouse_state.release(*button),
            },
            EventKind::KeyModifiersChanged { modifiers } => self.key_modifiers = *modifiers,
            EventKind::MouseMoved { position } => {
                self.mouse_position = *position;
            }
            _ => (),
        }
    }

    /// Clear the current frame state
    pub(crate) fn clear(&mut self) {
        self.mouse_state.clear();
        self.key_state.clear();
    }

    pub fn key_modifiers(&self) -> KeyModifiers {
        self.key_modifiers
    }
}
