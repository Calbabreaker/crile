#[derive(Debug)]
struct PaintJob {
    texture_id: egui::TextureId,
    index_alloc: crile::BufferAllocation,
    vertex_alloc: crile::BufferAllocation,
    index_count: u32,
    rect: crile::Rect,
}

pub struct EguiContext {
    ctx: Option<egui::Context>,
    raw_input: egui::RawInput,
    paint_jobs: Vec<PaintJob>,
    textures: hashbrown::HashMap<egui::TextureId, crile::RefId<crile::Texture>>,
    /// Used to scale the ui by the factor
    scale_factor: f32,
    /// Scale factor of the window so the user requested scale factor would be propertional to it
    window_scale: f32,
}

impl EguiContext {
    pub fn new(engine: &crile::Engine) -> Self {
        let mut egui = Self {
            ctx: Some(egui::Context::default()),
            window_scale: engine.window.scale_factor() as f32,
            scale_factor: 1.,
            raw_input: egui::RawInput {
                max_texture_side: Some(engine.gfx.wgpu.limits.max_texture_dimension_2d as usize),
                ..Default::default()
            },
            textures: hashbrown::HashMap::new(),
            paint_jobs: Vec::new(),
        };

        egui.set_ui_scale(1., engine.window.size());
        egui
    }

    #[must_use]
    pub fn begin_frame(&mut self, engine: &mut crile::Engine) -> egui::Context {
        self.paint_jobs.clear();

        self.raw_input.time = Some(engine.time.since_start() as f64);
        if to_egui_modifiers(engine.input.key_modifiers()).command {
            if engine.input.key_just_pressed(crile::KeyCode::KeyC) {
                self.push_event(egui::Event::Copy);
            } else if engine.input.key_just_pressed(crile::KeyCode::KeyX) {
                self.push_event(egui::Event::Cut);
            } else if engine.input.key_just_pressed(crile::KeyCode::KeyV) {
                self.push_event(egui::Event::Paste(engine.get_clipboard()));
            }
        }

        let ctx = self
            .ctx
            .take()
            .expect("tried to call egui begin frame before end frame or context is unintialized");
        ctx.set_pixels_per_point(self.scale_factor);
        ctx.begin_frame(self.raw_input.clone());
        ctx
    }

    pub fn end_frame(&mut self, engine: &mut crile::Engine, ctx: egui::Context) {
        let mut full_output = ctx.end_frame();
        let copied_text = full_output.platform_output.copied_text;
        if !copied_text.is_empty() {
            engine.set_clipboard(copied_text);
        }

        let icon = to_engine_cursor_icon(full_output.platform_output.cursor_icon);
        engine.window.set_cursor_icon(icon);

        let wgpu = &engine.gfx.wgpu;

        // Store all the texture egui is using
        while let Some((id, delta)) = full_output.textures_delta.set.pop() {
            let (pixels, width, height) = match &delta.image {
                egui::ImageData::Color(image) => {
                    let pixels = image
                        .pixels
                        .iter()
                        .flat_map(|pixel| pixel.to_array())
                        .collect::<Vec<_>>();
                    (pixels, image.width(), image.height())
                }
                egui::ImageData::Font(image) => {
                    let pixels = image
                        .srgba_pixels(None)
                        .flat_map(|pixel| pixel.to_array())
                        .collect::<Vec<_>>();
                    (pixels, image.width(), image.height())
                }
            };

            let size = glam::uvec2(width as u32, height as u32);

            if let Some(pos) = delta.pos {
                // Update the existing texture
                let texture = &self.textures[&id];
                let origin = glam::uvec2(pos[0] as u32, pos[1] as u32);
                texture.write_data(wgpu, origin, size, &pixels);
            } else {
                // Create new texture
                let texture = crile::Texture::from_pixels(wgpu, size, &pixels);
                self.textures.insert(id, texture.into());
            }
        }

        while let Some(id) = full_output.textures_delta.free.pop() {
            self.textures.remove(&id);
        }

        // Get all the paint jobs
        let clipped_primitives = ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
        for egui::ClippedPrimitive {
            primitive,
            clip_rect,
        } in clipped_primitives
        {
            match primitive {
                egui::epaint::Primitive::Mesh(mesh) => {
                    let vertices = mesh
                        .vertices
                        .iter()
                        .map(|v| crile::MeshVertex {
                            position: [v.pos.x, v.pos.y],
                            texture_coords: [v.uv.x, v.uv.y],
                            color: egui::Rgba::from(v.color).to_array(),
                        })
                        .collect::<Vec<_>>();

                    let rect = crile::Rect {
                        x: clip_rect.min.x,
                        y: clip_rect.min.y,
                        w: (clip_rect.max.x - clip_rect.min.x),
                        h: (clip_rect.max.y - clip_rect.min.y),
                    };

                    self.paint_jobs.push(PaintJob {
                        texture_id: mesh.texture_id,
                        vertex_alloc: engine
                            .gfx
                            .caches
                            .vertex_buffer_allocator
                            .alloc_write(wgpu, &vertices),
                        index_alloc: engine
                            .gfx
                            .caches
                            .index_buffer_allocator
                            .alloc_write(wgpu, &mesh.indices),
                        index_count: mesh.indices.len() as u32,
                        rect: rect * self.scale_factor,
                    })
                }
                egui::epaint::Primitive::Callback(_) => unimplemented!(),
            }
        }

        self.raw_input.events.clear();
        self.ctx = Some(ctx)
    }

    pub fn render<'a>(&'a mut self, render_pass: &mut crile::RenderPass<'a>) {
        render_pass.set_shader(render_pass.data.single_draw_shader.clone());
        render_pass.set_uniform(crile::DrawUniform {
            transform: render_pass.target_rect().matrix()
                * glam::Mat4::from_scale(glam::Vec3::splat(self.scale_factor)),
        });

        for job in &self.paint_jobs {
            render_pass.set_scissor_rect(job.rect);
            render_pass.set_texture(&self.textures[&job.texture_id]);
            render_pass.draw_mesh_single(crile::MeshView::new(
                job.vertex_alloc.as_slice(),
                job.index_alloc.as_slice(),
                job.index_count,
            ));
        }

        render_pass.reset_scissor_rect();
    }

    pub fn process_event(&mut self, engine: &crile::Engine, event: &crile::EventKind) {
        let mouse_position = to_egui_pos(engine.input.mouse_position() / self.scale_factor);
        let modifiers = to_egui_modifiers(engine.input.key_modifiers());

        match event {
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
                        crile::MouseButton::Other(0) => egui::PointerButton::Extra1,
                        crile::MouseButton::Other(1) => egui::PointerButton::Extra2,
                        _ => return,
                    },
                    modifiers,
                    pressed: state.is_pressed(),
                })
            }
            crile::EventKind::MouseScrolled { delta } => {
                self.push_event(egui::Event::Scroll(
                    egui::vec2(delta.x, delta.y) / self.scale_factor,
                ));
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
                self.set_ui_scale(ui_scale, engine.window.size());
            }
            crile::EventKind::WindowHoverChanged { hovering: false } => {
                self.push_event(egui::Event::PointerGone)
            }
            _ => (),
        }
    }

    /// Sets the scale of the ui propertional to the window scale factor
    pub fn set_ui_scale(&mut self, scale: f32, unscaled_size: glam::UVec2) {
        self.scale_factor = scale * self.window_scale;
        self.resize_event(&unscaled_size);
    }

    pub fn register_texture(&mut self, texture: &crile::RefId<crile::Texture>) -> egui::TextureId {
        let id = egui::TextureId::User(texture.id());
        self.textures.insert(id, texture.clone());
        id
    }

    /// Returns the viewport size of the egui context with the scale factor applied
    pub fn actual_size(&self) -> egui::Vec2 {
        self.raw_input.screen_rect.unwrap().size()
    }

    pub fn unregister_texture(&mut self, texture: &crile::RefId<crile::Texture>) {
        let id = egui::TextureId::User(texture.id());
        self.textures.remove(&id);
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
}

fn to_egui_pos(vec: glam::Vec2) -> egui::Pos2 {
    egui::pos2(vec.x, vec.y)
}

fn to_egui_modifiers(modifiers: crile::KeyModifiers) -> egui::Modifiers {
    egui::Modifiers {
        alt: modifiers.alt_key(),
        ctrl: modifiers.control_key(),
        shift: modifiers.shift_key(),
        mac_cmd: if cfg!(target_os = "macos") {
            modifiers.super_key()
        } else {
            false
        },
        command: if cfg!(target_os = "macos") {
            modifiers.super_key()
        } else {
            modifiers.control_key()
        },
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
        _ => return None,
    })
}

fn to_engine_cursor_icon(icon: egui::CursorIcon) -> Option<crile::CursorIcon> {
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
