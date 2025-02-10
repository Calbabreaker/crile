use crate::{egui_renderer::EguiRenderer, to_egui_pos};

pub struct EguiContext {
    ctx: Option<egui::Context>,
    raw_input: egui::RawInput,
    renderer: EguiRenderer,
    /// Used to scale the ui by the factor
    scale_factor: f32,
    /// Scale factor of the window so the user requested scale factor would be propertional to it
    window_scale: f32,
    target_window_id: crile::WindowId,
}

impl EguiContext {
    pub fn new(engine: &crile::Engine, target_window_id: crile::WindowId) -> Self {
        let window = engine
            .get_window(target_window_id)
            .expect("Invalid window ID");

        let mut context = Self {
            ctx: Some(egui::Context::default()),
            window_scale: window.scale_factor() as f32,
            scale_factor: 0.,
            raw_input: egui::RawInput {
                max_texture_side: Some(engine.gfx.wgpu.limits.max_texture_dimension_2d as usize),
                ..Default::default()
            },
            renderer: EguiRenderer::default(),
            target_window_id,
        };

        context.set_ui_scale(1., window.size());
        context
    }

    #[must_use]
    pub fn begin_frame(&mut self, engine: &mut crile::Engine) -> egui::Context {
        self.raw_input.time = Some(engine.time.elapsed().as_secs_f64());
        let input = &self.target_window(engine).input;
        let modifiers = to_egui_modifiers(input.key_modifiers());
        if modifiers.command {
            if input.key_just_pressed(crile::KeyCode::KeyC) {
                self.push_event(egui::Event::Copy);
            } else if input.key_just_pressed(crile::KeyCode::KeyX) {
                self.push_event(egui::Event::Cut);
            } else if input.key_just_pressed(crile::KeyCode::KeyV) {
                if let Some(text) = engine.clipboard.get() {
                    self.push_event(egui::Event::Paste(text));
                }
            }
        }

        self.raw_input.modifiers = modifiers;

        let ctx = self
            .ctx
            .take()
            .expect("Tried to call egui begin frame before end frame or multiple times");
        ctx.set_pixels_per_point(self.scale_factor);
        ctx.begin_pass(self.raw_input.clone());
        ctx
    }

    pub fn end_frame(&mut self, engine: &mut crile::Engine, ctx: egui::Context) {
        let full_output = ctx.end_pass();
        for command in &full_output.platform_output.commands {
            match command {
                egui::OutputCommand::CopyText(text) if !text.is_empty() => {
                    engine.clipboard.set(text.clone());
                }
                _ => (),
            }
        }

        self.target_window(engine)
            .set_cursor_icon(to_crile_cursor_icon(
                full_output.platform_output.cursor_icon,
            ));

        self.renderer.prepare(engine, &ctx, full_output);

        self.raw_input.events.clear();
        self.ctx = Some(ctx);
    }

    pub fn render<'a>(&'a mut self, render_pass: &mut crile::RenderPass<'a>) {
        self.renderer.render(render_pass, self.scale_factor);
    }

    pub fn process_event(&mut self, engine: &crile::Engine, event: &crile::Event) {
        if event.window_id != self.target_window_id {
            return;
        }

        let input = &self.target_window(engine).input;
        let mouse_position = to_egui_pos(input.mouse_position() / self.scale_factor);
        let modifiers = to_egui_modifiers(input.key_modifiers());

        match &event.kind {
            crile::EventKind::WindowResize { size } => self.resize_event(size),
            crile::EventKind::MouseMoved { .. } => {
                self.push_event(egui::Event::PointerMoved(mouse_position))
            }
            crile::EventKind::MouseInput { state, button } => {
                self.push_event(egui::Event::PointerButton {
                    pos: mouse_position,
                    button: match button {
                        crile::MouseButton::Left => egui::PointerButton::Primary,
                        crile::MouseButton::Right => egui::PointerButton::Secondary,
                        crile::MouseButton::Middle => egui::PointerButton::Middle,
                        crile::MouseButton::Back => egui::PointerButton::Extra1,
                        crile::MouseButton::Forward => egui::PointerButton::Extra2,
                        _ => return,
                    },
                    modifiers,
                    pressed: state.is_pressed(),
                })
            }
            crile::EventKind::MouseScrolled { delta } => {
                self.push_event(egui::Event::MouseWheel {
                    delta: egui::vec2(delta.x, delta.y) / self.scale_factor,
                    unit: egui::MouseWheelUnit::Point,
                    modifiers,
                });
            }
            crile::EventKind::KeyInput {
                state,
                repeat,
                keycode,
                text,
                ..
            } => {
                if state.is_pressed() && !text.is_empty() && text.chars().all(|c| !c.is_control()) {
                    self.push_event(egui::Event::Text(text.to_string()));
                }

                if let Some(key) = to_egui_key(*keycode) {
                    self.push_event(egui::Event::Key {
                        key,
                        physical_key: None,
                        pressed: state.is_pressed(),
                        repeat: *repeat,
                        modifiers,
                    });
                }
            }
            crile::EventKind::WindowFocusChanged { focused } => {
                self.push_event(egui::Event::WindowFocused(*focused))
            }
            crile::EventKind::WindowScaleChanged { factor } => {
                let ui_scale = self.scale_factor / self.window_scale;
                self.window_scale = *factor as f32;
                self.set_ui_scale(ui_scale, self.target_window(engine).size());
            }
            crile::EventKind::WindowHoverChanged { hovering: false } => {
                self.push_event(egui::Event::PointerGone)
            }
            _ => (),
        }
    }

    /// Sets the scale of the ui proportional to the window scale factor
    pub fn set_ui_scale(&mut self, scale: f32, unscaled_size: glam::UVec2) {
        self.scale_factor = scale * self.window_scale;
        self.resize_event(&unscaled_size);
    }

    #[must_use]
    pub fn register_texture(&mut self, texture: &crile::RefId<crile::Texture>) -> egui::TextureId {
        let id = egui::TextureId::User(texture.id());
        self.renderer.textures.insert(id, texture.clone());
        id
    }

    pub fn unregister_texture(&mut self, texture: &crile::RefId<crile::Texture>) {
        let id = egui::TextureId::User(texture.id());
        self.renderer.textures.remove(&id);
    }

    fn push_event(&mut self, event: egui::Event) {
        self.raw_input.events.push(event);
    }

    fn resize_event(&mut self, size: &glam::UVec2) {
        self.raw_input.screen_rect = Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(size.x as f32, size.y as f32) / self.scale_factor,
        ));
    }

    fn target_window<'a>(&self, engine: &'a crile::Engine) -> &'a crile::Window {
        engine
            .get_window(self.target_window_id)
            .expect("Invalid window Id")
    }
}

fn to_egui_modifiers(modifiers: crile::KeyModifiers) -> egui::Modifiers {
    egui::Modifiers {
        alt: modifiers.alt_key,
        ctrl: modifiers.control_key,
        shift: modifiers.shift_key,
        mac_cmd: if cfg!(target_os = "macos") {
            modifiers.super_key
        } else {
            false
        },
        command: modifiers.command_key(),
    }
}

fn to_egui_key(keycode: crile::KeyCode) -> Option<egui::Key> {
    Some(match keycode {
        crile::KeyCode::Escape => egui::Key::Escape,
        crile::KeyCode::Insert => egui::Key::Insert,
        crile::KeyCode::Home => egui::Key::Home,
        crile::KeyCode::Delete => egui::Key::Delete,
        crile::KeyCode::End => egui::Key::End,
        crile::KeyCode::PageDown => egui::Key::PageDown,
        crile::KeyCode::PageUp => egui::Key::PageUp,
        crile::KeyCode::ArrowLeft => egui::Key::ArrowLeft,
        crile::KeyCode::ArrowUp => egui::Key::ArrowUp,
        crile::KeyCode::ArrowRight => egui::Key::ArrowRight,
        crile::KeyCode::ArrowDown => egui::Key::ArrowDown,
        crile::KeyCode::Backspace => egui::Key::Backspace,
        crile::KeyCode::Enter => egui::Key::Enter,
        crile::KeyCode::Tab => egui::Key::Tab,
        crile::KeyCode::Space => egui::Key::Space,
        crile::KeyCode::KeyA => egui::Key::A,
        crile::KeyCode::KeyK => egui::Key::K,
        crile::KeyCode::KeyU => egui::Key::U,
        crile::KeyCode::KeyW => egui::Key::W,
        crile::KeyCode::KeyZ => egui::Key::Z,
        crile::KeyCode::KeyS => egui::Key::S,
        crile::KeyCode::KeyN => egui::Key::N,
        crile::KeyCode::KeyL => egui::Key::L,
        crile::KeyCode::KeyV => egui::Key::V,
        crile::KeyCode::KeyO => egui::Key::O,
        _ => return None,
    })
}

fn to_crile_cursor_icon(icon: egui::CursorIcon) -> Option<crile::CursorIcon> {
    Some(match icon {
        egui::CursorIcon::Default => crile::CursorIcon::Default,
        egui::CursorIcon::None => return None,
        egui::CursorIcon::ContextMenu => crile::CursorIcon::ContextMenu,
        egui::CursorIcon::Help => crile::CursorIcon::Help,
        egui::CursorIcon::PointingHand => crile::CursorIcon::Pointer,
        egui::CursorIcon::Progress => crile::CursorIcon::Progress,
        egui::CursorIcon::Wait => crile::CursorIcon::Wait,
        egui::CursorIcon::Cell => crile::CursorIcon::Cell,
        egui::CursorIcon::Crosshair => crile::CursorIcon::Crosshair,
        egui::CursorIcon::Text => crile::CursorIcon::Text,
        egui::CursorIcon::VerticalText => crile::CursorIcon::VerticalText,
        egui::CursorIcon::Alias => crile::CursorIcon::Alias,
        egui::CursorIcon::Copy => crile::CursorIcon::Copy,
        egui::CursorIcon::Move => crile::CursorIcon::Move,
        egui::CursorIcon::NoDrop => crile::CursorIcon::NoDrop,
        egui::CursorIcon::NotAllowed => crile::CursorIcon::NotAllowed,
        egui::CursorIcon::Grab => crile::CursorIcon::Grab,
        egui::CursorIcon::Grabbing => crile::CursorIcon::Grabbing,
        egui::CursorIcon::AllScroll => crile::CursorIcon::AllScroll,
        egui::CursorIcon::ResizeHorizontal => crile::CursorIcon::ColResize,
        egui::CursorIcon::ResizeNeSw => crile::CursorIcon::SwResize,
        egui::CursorIcon::ResizeNwSe => crile::CursorIcon::SeResize,
        egui::CursorIcon::ResizeVertical => crile::CursorIcon::RowResize,
        egui::CursorIcon::ResizeEast => crile::CursorIcon::EResize,
        egui::CursorIcon::ResizeSouthEast => crile::CursorIcon::SeResize,
        egui::CursorIcon::ResizeSouth => crile::CursorIcon::SResize,
        egui::CursorIcon::ResizeSouthWest => crile::CursorIcon::SwResize,
        egui::CursorIcon::ResizeWest => crile::CursorIcon::WResize,
        egui::CursorIcon::ResizeNorthWest => crile::CursorIcon::NwResize,
        egui::CursorIcon::ResizeNorth => crile::CursorIcon::NResize,
        egui::CursorIcon::ResizeNorthEast => crile::CursorIcon::NeResize,
        egui::CursorIcon::ResizeColumn => crile::CursorIcon::ColResize,
        egui::CursorIcon::ResizeRow => crile::CursorIcon::RowResize,
        egui::CursorIcon::ZoomIn => crile::CursorIcon::ZoomIn,
        egui::CursorIcon::ZoomOut => crile::CursorIcon::ZoomOut,
    })
}
