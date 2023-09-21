use crate::{Color, DrawUniform, Engine, Event, Mesh, MeshVertex, Rect, RenderPass, Texture};

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
    pub fn init(&mut self, engine: &mut Engine) {
        self.resize(engine.window.size());
    }

    pub fn update(&mut self, engine: &mut Engine, run_fn: impl FnOnce(&egui::Context)) {
        self.paint_jobs.clear();

        let mut full_output = self.ctx.run(self.raw_input.clone(), run_fn);
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
                            color: Color::from(v.color).to_array(),
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
    }

    pub fn draw<'a>(&'a mut self, render_pass: &mut RenderPass<'a>) {
        render_pass.set_shader(render_pass.data.single_draw_shader.clone());
        render_pass.set_uniform(DrawUniform {
            transform: glam::Mat4::orthographic_lh(
                0.,
                render_pass.target.size().x as f32,
                render_pass.target.size().y as f32,
                0.,
                0.,
                1.,
            ),
        });

        for paint_job in &self.paint_jobs {
            dbg!(paint_job.rect);
            render_pass.set_scissor_rect(paint_job.rect);
            render_pass.set_texture(&self.textures[&paint_job.texture_id]);
            render_pass.draw_mesh_single(&paint_job.mesh);
        }

        render_pass.reset_scissor_rect();
    }

    pub fn event(&mut self, event: &Event) {
        match event {
            Event::WindowResize { size } => self.resize(*size),
            _ => (),
        }
    }

    fn resize(&mut self, size: glam::UVec2) {
        self.raw_input.screen_rect = Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(size.x as f32, size.y as f32),
        ));
    }
}
