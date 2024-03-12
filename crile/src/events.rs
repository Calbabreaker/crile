use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;
pub use winit::{
    event::{ElementState as ButtonState, MouseButton},
    keyboard::{Key, KeyCode, ModifiersState as KeyModifiers},
    window::WindowId,
};

// We create our own event enum instead of using winit so we can manage it ourselves.
#[derive(Debug, PartialEq)]
pub enum EventKind {
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
    KeyModifiersChanged {
        modifiers: KeyModifiers,
    },
    WindowFocusChanged {
        focused: bool,
    },
    /// whenever the mouse leaves or exits the window
    WindowHoverChanged {
        hovering: bool,
    },
    WindowScaleChanged {
        factor: f64,
    },
    WindowClose,
    AppUpdate,
    AppRedraw,
}

#[derive(Debug)]
pub struct Event {
    pub kind: EventKind,
    pub window_id: Option<WindowId>,
}

pub(crate) fn convert_event(event: winit::event::Event<()>) -> Option<Event> {
    let kind = match event {
        winit::event::Event::AboutToWait => EventKind::AppUpdate,
        winit::event::Event::WindowEvent { ref event, .. } => match event {
            winit::event::WindowEvent::RedrawRequested => EventKind::AppRedraw,
            winit::event::WindowEvent::CloseRequested | winit::event::WindowEvent::Destroyed => {
                EventKind::WindowClose
            }
            winit::event::WindowEvent::Resized(size) => EventKind::WindowResize {
                size: glam::UVec2::new(size.width, size.height),
            },
            winit::event::WindowEvent::ModifiersChanged(modifiers) => {
                EventKind::KeyModifiersChanged {
                    modifiers: modifiers.state(),
                }
            }
            winit::event::WindowEvent::KeyboardInput { event, .. } => EventKind::KeyInput {
                state: event.state,
                key: event.logical_key.clone(),
                keycode: match event.physical_key {
                    winit::keyboard::PhysicalKey::Code(c) => c,
                    winit::keyboard::PhysicalKey::Unidentified(_) => KeyCode::F35,
                },
                repeat: event.repeat,
                text: event.text_with_all_modifiers().unwrap_or("").to_string(),
            },
            winit::event::WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                EventKind::WindowScaleChanged {
                    factor: *scale_factor,
                }
            }
            winit::event::WindowEvent::MouseInput { state, button, .. } => EventKind::MouseInput {
                state: *state,
                button: *button,
            },
            winit::event::WindowEvent::CursorMoved { position, .. } => EventKind::MouseMoved {
                position: glam::Vec2::new(position.x as f32, position.y as f32),
            },
            winit::event::WindowEvent::MouseWheel { delta, .. } => match delta {
                winit::event::MouseScrollDelta::LineDelta(x, y) => EventKind::MouseScrolled {
                    delta: glam::Vec2::new(*x, *y),
                },
                winit::event::MouseScrollDelta::PixelDelta(pos) => EventKind::MouseScrolled {
                    delta: glam::Vec2::new(pos.x as f32, pos.y as f32),
                },
            },
            winit::event::WindowEvent::Focused(focused) => {
                EventKind::WindowFocusChanged { focused: *focused }
            }
            winit::event::WindowEvent::CursorLeft { .. } => {
                EventKind::WindowHoverChanged { hovering: false }
            }
            winit::event::WindowEvent::CursorEntered { .. } => {
                EventKind::WindowHoverChanged { hovering: true }
            }
            _ => return None,
        },
        _ => return None,
    };

    let window_id = match event {
        winit::event::Event::WindowEvent { window_id, .. } => Some(window_id),
        _ => None,
    };

    Some(Event { kind, window_id })
}
