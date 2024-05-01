mod codes;
mod input;

use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;
pub use winit::window::WindowId;

pub use codes::*;
pub use input::*;

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
    FileDropped {
        path: std::path::PathBuf,
    },
    FileHovered {
        path: std::path::PathBuf,
    },
}

#[derive(Debug)]
pub struct Event {
    pub kind: EventKind,
    pub window_id: Option<WindowId>,
}

impl Event {
    pub(crate) fn from_winit_window_event(
        window_id: WindowId,
        event: winit::event::WindowEvent,
    ) -> Option<Event> {
        let kind = match event {
            winit::event::WindowEvent::CloseRequested | winit::event::WindowEvent::Destroyed => {
                EventKind::WindowClose
            }
            winit::event::WindowEvent::Resized(size) => EventKind::WindowResize {
                size: glam::UVec2::new(size.width, size.height),
            },
            winit::event::WindowEvent::ModifiersChanged(modifiers) => {
                EventKind::KeyModifiersChanged {
                    modifiers: KeyModifiers::from_winit(modifiers.state()),
                }
            }
            winit::event::WindowEvent::KeyboardInput { event, .. } => EventKind::KeyInput {
                state: ButtonState::from_winit(event.state),
                text: event.text_with_all_modifiers().unwrap_or("").to_string(),
                keycode: match event.physical_key {
                    winit::keyboard::PhysicalKey::Code(c) => KeyCode::from_winit(c),
                    winit::keyboard::PhysicalKey::Unidentified(_) => KeyCode::Unknown,
                },
                repeat: event.repeat,
            },
            winit::event::WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                EventKind::WindowScaleChanged {
                    factor: scale_factor,
                }
            }
            winit::event::WindowEvent::MouseInput { state, button, .. } => EventKind::MouseInput {
                state: ButtonState::from_winit(state),
                button: MouseButton::from_winit(button),
            },
            winit::event::WindowEvent::CursorMoved { position, .. } => EventKind::MouseMoved {
                position: glam::Vec2::new(position.x as f32, position.y as f32),
            },
            winit::event::WindowEvent::MouseWheel { delta, .. } => match delta {
                winit::event::MouseScrollDelta::LineDelta(x, y) => EventKind::MouseScrolled {
                    delta: glam::Vec2::new(x, y),
                },
                winit::event::MouseScrollDelta::PixelDelta(pos) => EventKind::MouseScrolled {
                    delta: glam::Vec2::new(pos.x as f32, pos.y as f32),
                },
            },
            winit::event::WindowEvent::Focused(focused) => {
                EventKind::WindowFocusChanged { focused }
            }
            winit::event::WindowEvent::CursorLeft { .. } => {
                EventKind::WindowHoverChanged { hovering: false }
            }
            winit::event::WindowEvent::CursorEntered { .. } => {
                EventKind::WindowHoverChanged { hovering: true }
            }
            winit::event::WindowEvent::DroppedFile(path) => EventKind::FileDropped { path },
            winit::event::WindowEvent::HoveredFile(path) => EventKind::FileHovered { path },
            _ => return None,
        };

        Some(Event {
            kind,
            window_id: Some(window_id),
        })
    }
}
