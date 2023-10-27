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
}

impl EguiContext {
    pub fn new(engine: &crile::Engine) -> Self {
        Self {
            ctx: Some(egui::Context::default()),
            raw_input: egui::RawInput {
                max_texture_side: Some(engine.gfx.wgpu.limits.max_texture_dimension_2d as usize),
                screen_rect: rect_from_size(engine.window.size()),
                ..Default::default()
            },
            paint_jobs: Vec::new(),
            textures: hashbrown::HashMap::new(),
        }
    }

    pub fn begin_frame(&mut self, engine: &mut crile::Engine) -> egui::Context {
        self.paint_jobs.clear();

        if to_egui_modifiers(engine.input.key_modifiers()).command {
            if engine.input.key_just_pressed(crile::KeyCode::KeyC) {
                self.push_event(egui::Event::Copy);
            } else if engine.input.key_just_pressed(crile::KeyCode::KeyX) {
                self.push_event(egui::Event::Cut);
            } else if engine.input.key_just_pressed(crile::KeyCode::KeyV) {
                self.push_event(egui::Event::Paste(engine.window.get_clipboard()));
            }
        }

        let ctx = self
            .ctx
            .take()
            .expect("tried to call egui begine frame twice");
        ctx.begin_frame(self.raw_input.clone());
        ctx
    }

    pub fn end_frame(&mut self, engine: &mut crile::Engine, ctx: egui::Context) {
        let mut full_output = ctx.end_frame();
        let copied_text = full_output.platform_output.copied_text;
        if !copied_text.is_empty() {
            engine.window.set_clipboard(copied_text);
        }

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

            let texture = crile::Texture::from_pixels(wgpu, width as u32, height as u32, &pixels);
            self.textures.insert(id, texture.into());
        }

        while let Some(id) = full_output.textures_delta.free.pop() {
            self.textures.remove(&id);
        }

        // Get all the paint jobs
        let clipped_primitives = ctx.tessellate(full_output.shapes);
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
                        rect: crile::Rect {
                            x: clip_rect.min.x,
                            y: clip_rect.min.y,
                            w: (clip_rect.max.x - clip_rect.min.x),
                            h: (clip_rect.max.y - clip_rect.min.y),
                        },
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
            transform: render_pass.target_rect().matrix(),
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

    pub fn event(&mut self, engine: &crile::Engine, event: &crile::Event) {
        let mouse_position = to_egui_pos(engine.input.mouse_position());
        let modifiers = to_egui_modifiers(engine.input.key_modifiers());

        match event {
            crile::Event::WindowResize { size } => {
                self.raw_input.screen_rect = rect_from_size(*size)
            }
            crile::Event::MouseMoved { .. } => {
                self.push_event(egui::Event::PointerMoved(mouse_position))
            }
            crile::Event::MouseInput { state, button } => {
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
            crile::Event::MouseScrolled { delta } => {
                self.push_event(egui::Event::Scroll(egui::vec2(delta.x, delta.y)))
            }
            crile::Event::KeyInput {
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
                        pressed: state.is_pressed(),
                        repeat: *repeat,
                        modifiers,
                    });
                }
            }
            crile::Event::WindowFocusChanged { focused } => {
                self.push_event(egui::Event::WindowFocused(*focused))
            }
            crile::Event::MouseHoverChanged { hovering: false } => {
                self.push_event(egui::Event::PointerGone)
            }
            _ => (),
        }
    }

    fn push_event(&mut self, event: egui::Event) {
        self.raw_input.events.push(event);
    }

    pub fn register_texture(&mut self, texture: &crile::RefId<crile::Texture>) -> egui::TextureId {
        let id = egui::TextureId::User(texture.id());
        self.textures.insert(id, texture.clone());
        id
    }

    pub fn unregister_texture(&mut self, texture: &crile::RefId<crile::Texture>) {
        let id = egui::TextureId::User(texture.id());
        self.textures.remove(&id);
    }
}

fn rect_from_size(size: glam::UVec2) -> Option<egui::Rect> {
    Some(egui::Rect::from_min_size(
        egui::Pos2::ZERO,
        egui::vec2(size.x as f32, size.y as f32),
    ))
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

