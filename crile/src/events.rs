use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;
pub use winit::{
    event::{ElementState as ButtonState, MouseButton},
    keyboard::{Key, KeyCode, ModifiersState as KeyModifiers},
};

// We create our own event enum instead of using winit so we can manage it ourselves.
#[derive(Debug, PartialEq)]
pub enum Event {
    WindowResize {
        size: glam::UVec2,
    },
    MouseInput {
        state: ButtonState,
        button: MouseButton,
    },
    MouseMoved {
        position: glam::Vec2,
    },
    MouseScrolled {
        delta: glam::Vec2,
    },
    KeyInput {
        state: ButtonState,
        /// Whether or not the key is a repeating key
        repeat: bool,
        /// Represents the localized key in the keyboard layout
        key: Key,
        /// Represents the physical key on the keyboard not accounting for keyboard layouts
        keycode: KeyCode,
        text: String,
    },
    ModifiersChanged {
        modifiers: KeyModifiers,
    },
    WindowFocusChanged {
        focused: bool,
    },
    /// Sent wheneever the mouse leaves or exits the window
    MouseHoverChanged {
        hovering: bool,
    },
    WindowClose,
}

pub(crate) fn convert_event(event: winit::event::Event<()>) -> Option<Event> {
    Some(match event {
        winit::event::Event::WindowEvent { ref event, .. } => match event {
            winit::event::WindowEvent::CloseRequested => Event::WindowClose,
            winit::event::WindowEvent::Resized(size) => Event::WindowResize {
                size: glam::UVec2::new(size.width, size.height),
            },
            winit::event::WindowEvent::ModifiersChanged(modifiers) => Event::ModifiersChanged {
                modifiers: modifiers.state(),
            },
            winit::event::WindowEvent::KeyboardInput { event, .. } => Event::KeyInput {
                state: event.state,
                key: event.logical_key.clone(),
                keycode: match event.physical_key {
                    winit::keyboard::PhysicalKey::Code(c) => c,
                    winit::keyboard::PhysicalKey::Unidentified(_) => KeyCode::F35,
                },
                repeat: event.repeat,
                text: event.text_with_all_modifiers().unwrap_or("").to_string(),
            },
            winit::event::WindowEvent::MouseInput { state, button, .. } => Event::MouseInput {
                state: *state,
                button: *button,
            },
            winit::event::WindowEvent::CursorMoved { position, .. } => Event::MouseMoved {
                position: glam::Vec2::new(position.x as f32, position.y as f32),
            },
            winit::event::WindowEvent::MouseWheel { delta, .. } => match delta {
                winit::event::MouseScrollDelta::LineDelta(x, y) => Event::MouseScrolled {
                    delta: glam::Vec2::new(*x, *y),
                },
                winit::event::MouseScrollDelta::PixelDelta(pos) => Event::MouseScrolled {
                    delta: glam::Vec2::new(pos.x as f32, pos.y as f32),
                },
            },
            winit::event::WindowEvent::Focused(focused) => {
                Event::WindowFocusChanged { focused: *focused }
            }
            winit::event::WindowEvent::CursorLeft { .. } => {
                Event::MouseHoverChanged { hovering: false }
            }
            winit::event::WindowEvent::CursorEntered { .. } => {
                Event::MouseHoverChanged { hovering: true }
            }
            _ => return None,
        },
        _ => return None,
    })
}
