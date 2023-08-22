pub use winit::event::{ElementState, MouseButton, VirtualKeyCode};

use crate::Vector2;

#[derive(Debug)]
pub enum Event {
    WindowResize {
        size: Vector2,
    },
    MouseInput {
        state: ElementState,
        button: MouseButton,
    },
    KeyboardInput {
        state: ElementState,
        keycode: VirtualKeyCode,
    },
    WindowClose,
    Unknown,
}

pub(crate) fn process_event(event: winit::event::Event<()>) -> Event {
    match event {
        winit::event::Event::WindowEvent { ref event, .. } => match event {
            winit::event::WindowEvent::CloseRequested => Event::WindowClose,
            winit::event::WindowEvent::Resized(size) => Event::WindowResize {
                size: Vector2::new(size.width as f32, size.height as f32),
            },
            winit::event::WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                Event::WindowResize {
                    size: Vector2::new(new_inner_size.width as f32, new_inner_size.height as f32),
                }
            }
            winit::event::WindowEvent::KeyboardInput {
                input:
                    winit::event::KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => Event::KeyboardInput {
                state: *state,
                keycode: *keycode,
            },
            winit::event::WindowEvent::MouseInput { state, button, .. } => Event::MouseInput {
                state: *state,
                button: *button,
            },
            _ => Event::Unknown,
        },
        _ => Event::Unknown,
    }
}
