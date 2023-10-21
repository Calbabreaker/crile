use crate::{
    DrawUniform, Engine, Event, KeyCode, KeyModifiers, Mesh, MeshVertex, MouseButton, Rect,
    RenderPass, Texture,
};

#[derive(Debug)]
struct PaintJob {
    texture_id: egui::TextureId,
    mesh: Mesh,
    rect: Rect,
}

#[derive(Default)]
pub struct EguiContext {
    ctx: egui::Context,
    raw_input: egui::RawInput,
    paint_jobs: Vec<PaintJob>,
    textures: hashbrown::HashMap<egui::TextureId, Texture>,
}

impl EguiContext {
    pub fn init(&mut self, engine: &Engine) {
        self.raw_input.max_texture_side =
            Some(engine.gfx.wgpu.limits.max_texture_dimension_2d as usize);
        self.resize(engine.window.size());
    }

    pub fn update(
        &mut self,
        engine: &mut Engine,
        run_fn: impl FnOnce(&egui::Context, &mut Engine),
    ) {
        self.paint_jobs.clear();

        if to_egui_modifiers(engine.input.key_modifiers()).command {
            if engine.input.key_just_pressed(KeyCode::KeyC) {
                self.push_event(egui::Event::Copy);
            } else if engine.input.key_just_pressed(KeyCode::KeyX) {
                self.push_event(egui::Event::Cut);
            } else if engine.input.key_just_pressed(KeyCode::KeyV) {
                self.push_event(egui::Event::Paste(engine.window.get_clipboard()));
            }
        }

        let mut full_output = self
            .ctx
            .run(self.raw_input.clone(), |ctx| run_fn(ctx, engine));
        let copied_text = full_output.platform_output.copied_text;
        if !copied_text.is_empty() {
            engine.window.set_clipboard(copied_text);
        }

        let wgpu = &engine.gfx.wgpu;

        // Store all the texture egui is using
        while let Some((id, delta)) = full_output.textures_delta.set.pop() {
            let texture = match &delta.image {
                egui::ImageData::Color(image) => {
                    let pixels = image
                        .pixels
                        .iter()
                        .flat_map(|pixel| pixel.to_array())
                        .collect::<Vec<_>>();
                    Texture::new(wgpu, image.width() as u32, image.height() as u32, &pixels)
                }
                egui::ImageData::Font(image) => {
                    let pixels = image
                        .srgba_pixels(None)
                        .flat_map(|pixel| pixel.to_array())
                        .collect::<Vec<_>>();
                    Texture::new(wgpu, image.width() as u32, image.height() as u32, &pixels)
                }
            };

            self.textures.insert(id, texture);
        }

        while let Some(id) = full_output.textures_delta.free.pop() {
            self.textures.remove(&id);
        }

        // Get all the paint jobs
        let clipped_primitives = self.ctx.tessellate(full_output.shapes);
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
                        .map(|v| MeshVertex {
                            position: [v.pos.x, v.pos.y],
                            texture_coords: [v.uv.x, v.uv.y],
                            color: egui::Rgba::from(v.color).to_array(),
                        })
                        .collect::<Vec<_>>();

                    self.paint_jobs.push(PaintJob {
                        texture_id: mesh.texture_id,
                        mesh: Mesh::new(wgpu, &vertices, &mesh.indices),
                        rect: Rect {
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
    }

    pub fn render<'a>(&'a mut self, render_pass: &mut RenderPass<'a>) {
        render_pass.set_shader(render_pass.data.single_draw_shader.clone());
        render_pass.set_uniform(DrawUniform {
            transform: render_pass.target_rect().matrix(),
        });

        for paint_job in &self.paint_jobs {
            render_pass.set_scissor_rect(paint_job.rect);
            render_pass.set_texture(&self.textures[&paint_job.texture_id]);
            render_pass.draw_mesh_single(&paint_job.mesh);
        }

        render_pass.reset_scissor_rect();
    }

    pub fn event(&mut self, engine: &Engine, event: &Event) {
        let mouse_position = to_egui_pos(engine.input.mouse_position());
        let modifiers = to_egui_modifiers(engine.input.key_modifiers());

        match event {
            Event::WindowResize { size } => self.resize(*size),
            Event::MouseMoved { .. } => self.push_event(egui::Event::PointerMoved(mouse_position)),
            Event::MouseInput { state, button } => self.push_event(egui::Event::PointerButton {
                pos: mouse_position,
                button: match button {
                    MouseButton::Left => egui::PointerButton::Primary,
                    MouseButton::Right => egui::PointerButton::Secondary,
                    MouseButton::Middle => egui::PointerButton::Middle,
                    MouseButton::Other(0) => egui::PointerButton::Extra1,
                    MouseButton::Other(1) => egui::PointerButton::Extra2,
                    _ => return,
                },
                modifiers,
                pressed: state.is_pressed(),
            }),
            Event::MouseScrolled { delta } => {
                self.push_event(egui::Event::Scroll(egui::vec2(delta.x, delta.y)))
            }
            Event::KeyInput {
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
            Event::WindowFocusChanged { focused } => {
                self.push_event(egui::Event::WindowFocused(*focused))
            }
            Event::MouseHoverChanged { hovering: false } => {
                self.push_event(egui::Event::PointerGone)
            }
            _ => (),
        }
    }

    fn push_event(&mut self, event: egui::Event) {
        self.raw_input.events.push(event);
    }

    fn resize(&mut self, size: glam::UVec2) {
        self.raw_input.screen_rect = Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(size.x as f32, size.y as f32),
        ));
    }
}

fn to_egui_pos(vec: glam::Vec2) -> egui::Pos2 {
    egui::pos2(vec.x, vec.y)
}

fn to_egui_modifiers(modifiers: KeyModifiers) -> egui::Modifiers {
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

fn to_egui_key(keycode: KeyCode) -> Option<egui::Key> {
    Some(match keycode {
        KeyCode::Escape => egui::Key::Escape,
        KeyCode::Insert => egui::Key::Insert,
        KeyCode::Home => egui::Key::Home,
        KeyCode::Delete => egui::Key::Delete,
        KeyCode::End => egui::Key::End,
        KeyCode::PageDown => egui::Key::PageDown,
        KeyCode::PageUp => egui::Key::PageUp,
        KeyCode::ArrowLeft => egui::Key::ArrowLeft,
        KeyCode::ArrowUp => egui::Key::ArrowUp,
        KeyCode::ArrowRight => egui::Key::ArrowRight,
        KeyCode::ArrowDown => egui::Key::ArrowDown,
        KeyCode::Backspace => egui::Key::Backspace,
        KeyCode::Enter => egui::Key::Enter,
        KeyCode::Tab => egui::Key::Tab,
        KeyCode::Space => egui::Key::Space,
        KeyCode::KeyA => egui::Key::A,
        KeyCode::KeyK => egui::Key::K,
        KeyCode::KeyU => egui::Key::U,
        KeyCode::KeyW => egui::Key::W,
        KeyCode::KeyZ => egui::Key::Z,
        _ => return None,
    })
}
