struct PaintJob {
    texture_id: egui::TextureId,
    index_alloc: crile::BufferAllocation,
    vertex_alloc: crile::BufferAllocation,
    index_count: u32,
    clip_rect: crile::Rect,
}

#[derive(Default)]
pub(crate) struct EguiRenderer {
    paint_jobs: Vec<PaintJob>,
    pub textures: hashbrown::HashMap<egui::TextureId, crile::RefId<crile::Texture>>,
}

impl EguiRenderer {
    pub fn prepare(
        &mut self,
        engine: &mut crile::Engine,
        ctx: &egui::Context,
        full_output: egui::FullOutput,
    ) {
        // Store all the texture egui is using
        for (id, delta) in &full_output.textures_delta.set {
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
                let texture = &self.textures[id];
                let origin = glam::uvec2(pos[0] as u32, pos[1] as u32);
                texture.write_data(&engine.gfx.wgpu, origin, size, &pixels);
            } else {
                // Create new texture
                let texture = crile::Texture::from_pixels(&engine.gfx.wgpu, size, &pixels);
                self.textures.insert(*id, texture.into());
            }
        }

        for id in full_output.textures_delta.free {
            self.textures.remove(&id);
        }

        // Get all the paint jobs
        self.paint_jobs.clear();
        let clipped_primitives = ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
        for egui::ClippedPrimitive {
            primitive,
            clip_rect,
        } in clipped_primitives
        {
            match primitive {
                egui::epaint::Primitive::Mesh(mesh) => self.add_paint_job(engine, mesh, clip_rect),
                egui::epaint::Primitive::Callback(_) => unimplemented!(),
            }
        }
    }

    pub fn render<'a>(&'a mut self, render_pass: &mut crile::RenderPass<'a>, scale_factor: f32) {
        render_pass.set_shader(render_pass.data.single_draw_shader.clone());
        render_pass.set_uniform(crile::DrawUniform {
            transform: render_pass.target_rect().matrix()
                * glam::Mat4::from_scale(glam::Vec3::splat(scale_factor)),
        });

        for job in &self.paint_jobs {
            render_pass.set_scissor_rect(job.clip_rect * scale_factor);
            render_pass.set_texture(&self.textures[&job.texture_id]);
            render_pass.draw_mesh_single(crile::MeshView::new(
                job.vertex_alloc.as_slice(),
                job.index_alloc.as_slice(),
                job.index_count,
            ));
        }

        render_pass.reset_scissor_rect();
    }

    fn add_paint_job(
        &mut self,
        engine: &mut crile::Engine,
        mesh: egui::Mesh,
        clip_rect: egui::Rect,
    ) {
        // Convert to crile vertices
        let vertices = mesh
            .vertices
            .iter()
            .map(|v| crile::MeshVertex {
                position: [v.pos.x, v.pos.y],
                texture_coords: [v.uv.x, v.uv.y],
                color: egui::Rgba::from(v.color).to_array(),
            })
            .collect::<Vec<_>>();

        let wgpu = &engine.gfx.wgpu;
        let caches = &mut engine.gfx.caches;

        self.paint_jobs.push(PaintJob {
            texture_id: mesh.texture_id,
            vertex_alloc: caches.vertex_buffer_allocator.alloc_write(wgpu, &vertices),
            index_alloc: caches
                .index_buffer_allocator
                .alloc_write(wgpu, &mesh.indices),
            index_count: mesh.indices.len() as u32,
            clip_rect: crile::Rect {
                x: clip_rect.min.x,
                y: clip_rect.min.y,
                w: (clip_rect.max.x - clip_rect.min.x),
                h: (clip_rect.max.y - clip_rect.min.y),
            },
        })
    }
}
