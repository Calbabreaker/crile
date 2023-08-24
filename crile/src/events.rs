pub use winit::{
    event::{ElementState as ButtonState, MouseButton},
    keyboard::{Key, KeyCode},
};

use crate::{Vector2, Vector2U};

// We create our own event enum instead of using winit so we can manage it ourselves.
#[derive(Debug, PartialEq)]
pub enum Event {
    WindowResize {
        size: Vector2U,
    },
    MouseInput {
        state: ButtonState,
        button: MouseButton,
    },
    MouseMoved {
        position: Vector2,
    },
    MouseScrolled {
        delta: Vector2,
    },
    KeyInput {
        state: ButtonState,
        /// Whether or not the key is a repeating key
        repeat: bool,
        /// Represents the localized key in the keyboard layout
        key: Key,
        /// Represents the physical key on the keyboard not accounting for keyboard layouts
        keycode: KeyCode,
    },
    ApplicationUpdate,
    WindowClose,
}

pub(crate) fn convert_event(event: winit::event::Event<()>) -> Option<Event> {
    Some(match event {
        winit::event::Event::AboutToWait => Event::ApplicationUpdate,
        winit::event::Event::WindowEvent { ref event, .. } => match event {
            winit::event::WindowEvent::CloseRequested => Event::WindowClose,
            winit::event::WindowEvent::Resized(size) => Event::WindowResize {
                size: Vector2U::new(size.width, size.height),
            },
            winit::event::WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        state,
                        logical_key,
                        physical_key,
                        repeat,
                        ..
                    },
                ..
            } => Event::KeyInput {
                state: *state,
                key: logical_key.clone(),
                keycode: *physical_key,
                repeat: *repeat,
            },
            winit::event::WindowEvent::MouseInput { state, button, .. } => Event::MouseInput {
                state: *state,
                button: *button,
            },
            winit::event::WindowEvent::CursorMoved { position, .. } => Event::MouseMoved {
                position: Vector2::new(position.x as f32, position.y as f32),
            },
            winit::event::WindowEvent::MouseWheel { delta, .. } => match delta {
                winit::event::MouseScrollDelta::LineDelta(x, y) => Event::MouseScrolled {
                    delta: Vector2::new(*x, *y),
                },
                winit::event::MouseScrollDelta::PixelDelta(pos) => Event::MouseScrolled {
                    delta: Vector2::new(pos.x as f32, pos.y as f32),
                },
            },
            _ => None?,
        },
        _ => None?,
    })
}
