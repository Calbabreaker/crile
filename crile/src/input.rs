use core::hash::Hash;

use crate::{ButtonState, Event, KeyCode, KeyModifiers, MouseButton};

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

    /// Update an internal state with crile::Event
    pub fn process_event(&mut self, event: &Event) {
        match event {
            Event::KeyInput {
                keycode,
                state,
                repeat: false,
                ..
            } => match state {
                ButtonState::Pressed => self.key_state.press(*keycode),
                ButtonState::Released => self.key_state.release(*keycode),
            },
            Event::MouseInput { button, state } => match state {
                ButtonState::Pressed => self.mouse_state.press(*button),
                ButtonState::Released => self.mouse_state.release(*button),
            },
            Event::ModifiersChanged { modifiers } => self.key_modifiers = *modifiers,
            Event::MouseMoved { position } => {
                self.mouse_position = *position;
            }
            _ => (),
        }
    }

    /// Clear one frame only internal state from current frame
    pub fn clear(&mut self) {
        self.mouse_state.clear();
        self.key_state.clear();
    }

    pub fn key_modifiers(&self) -> KeyModifiers {
        self.key_modifiers
    }
}
