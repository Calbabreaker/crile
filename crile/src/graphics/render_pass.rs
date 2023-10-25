use std::num::NonZeroU64;

use super::{
    BindGroupEntries, Color, GraphicsCaches, GraphicsContext, GraphicsData, MeshVertex, Rect,
    RenderPipelineConfig, Shader, ShaderKind, Texture, TextureRef, WGPUContext,
};
use crate::{MeshRef, RefId};

#[repr(C)]
#[derive(Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DrawUniform {
    pub transform: glam::Mat4,
}

#[repr(C)]
#[derive(Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RenderInstance {
    pub transform: glam::Mat4,
    pub color: Color,
}

/// Wrapper around a wgpu::RenderPass with a higher level api
/// Automatically caches pipelines, bind groups and dynamic buffers
pub struct RenderPass<'a> {
    gpu_render_pass: wgpu::RenderPass<'a>,
    shader: RefId<Shader>,
    dirty_pipline: bool,
    blend_mode: wgpu::BlendState,

    pub target: TextureRef<'a>,
    wgpu: &'a WGPUContext,
    caches: &'a mut GraphicsCaches,
    pub data: &'a GraphicsData,
}

impl<'a> RenderPass<'a> {
    /// Creates a new render pass from the current frame (this can only be created in the render function)
    /// target is the texture to render to, set to None to use the surface texture
    pub fn new(
        gfx: &'a mut GraphicsContext,
        clear_color: Option<Color>,
        target: Option<TextureRef<'a>>,
    ) -> Self {
        let frame = gfx
            .frame
            .as_mut()
            .expect("tried to create render pass but frame doesn't exist");

        let target = target.unwrap_or(TextureRef::new(&frame.output.texture, &frame.output_view));
        let gpu_render_pass = frame
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target.gpu_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: match clear_color {
                            None => wgpu::LoadOp::Load,
                            Some(c) => wgpu::LoadOp::Clear(c.into()),
                        },
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

        Self {
            gpu_render_pass,
            shader: gfx.data.single_draw_shader.clone(),
            dirty_pipline: true,
            target,
            blend_mode: wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING,

            // We need to get references to WGPUContext, GfxData, GfxCaches seperately or rust is
            // going to complain about mutiple borrows
            wgpu: &gfx.wgpu,
            caches: &mut gfx.caches,
            data: &gfx.data,
        }
    }

    pub fn draw_mesh_instanced(&mut self, mesh: MeshRef<'a>, instances: &[RenderInstance]) {
        let instances_alloc = self
            .caches
            .storage_buffer_allocator
            .alloc_write(self.wgpu, instances);

        let instances_bind_group = self.caches.bind_group.get(
            self.wgpu,
            &BindGroupEntries::new().buffer(
                wgpu::ShaderStages::VERTEX,
                &instances_alloc.buffer,
                wgpu::BufferBindingType::Storage { read_only: true },
                NonZeroU64::new(instances_alloc.size),
                true,
            ),
        );

        self.set_bind_group(2, instances_bind_group, &[instances_alloc.offset as u32]);
        self.draw_mesh(mesh, instances.len() as u32);
    }

    pub fn draw_mesh_single(&mut self, mesh: MeshRef<'a>) {
        self.draw_mesh(mesh, 1);
    }

    fn draw_mesh(&mut self, mesh: MeshRef<'a>, instance_count: u32) {
        self.update_pipeline();
        self.gpu_render_pass
            .set_index_buffer(mesh.index_buffer, wgpu::IndexFormat::Uint32);
        self.gpu_render_pass
            .set_vertex_buffer(0, mesh.vertex_buffer);
        self.gpu_render_pass
            .draw_indexed(0..mesh.index_count, 0, 0..instance_count);
    }

    pub fn set_texture(&mut self, texture: &Texture) {
        let sampler = self.caches.sampler.get(self.wgpu, texture.sampler_config);
        let texture_bind_group = self.caches.bind_group.get(
            self.wgpu,
            &BindGroupEntries::new()
                .texture(wgpu::ShaderStages::FRAGMENT, &texture.gpu_view)
                .sampler(wgpu::ShaderStages::FRAGMENT, &sampler),
        );

        self.set_bind_group(1, texture_bind_group, &[]);
    }

    pub fn set_uniform<U: bytemuck::Pod>(&mut self, uniform: U) {
        let uniform_alloc = self
            .caches
            .uniform_buffer_allocator
            .alloc_write(self.wgpu, &[uniform]);

        let uniform_bind_group = self.caches.bind_group.get(
            self.wgpu,
            &BindGroupEntries::new().buffer(
                wgpu::ShaderStages::VERTEX,
                &uniform_alloc.buffer,
                wgpu::BufferBindingType::Uniform,
                NonZeroU64::new(uniform_alloc.size),
                true,
            ),
        );

        self.set_bind_group(0, uniform_bind_group, &[uniform_alloc.offset as u32]);
    }

    /// Set a shader from self.data or a custom shader
    pub fn set_shader(&mut self, shader: RefId<Shader>) {
        if shader != self.shader {
            self.shader = shader;
            self.dirty_pipline = true;
        }
    }

    fn update_pipeline(&mut self) {
        // Only update the pipeline when something changed
        if !self.dirty_pipline {
            return;
        }

        let uniform_layout = self.caches.bind_group.get_layout(
            self.wgpu,
            &BindGroupEntries::new().buffer_layout(
                wgpu::ShaderStages::VERTEX,
                wgpu::BufferBindingType::Uniform,
                true,
            ),
        );

        let texture_layout = self.caches.bind_group.get_layout(
            self.wgpu,
            &BindGroupEntries::new()
                .texture_layout(wgpu::ShaderStages::FRAGMENT)
                .sampler_layout(wgpu::ShaderStages::FRAGMENT),
        );

        let mut layouts = vec![uniform_layout, texture_layout];

        if self.shader.kind == ShaderKind::Instanced {
            let instances_layout = self.caches.bind_group.get_layout(
                self.wgpu,
                &BindGroupEntries::new().buffer_layout(
                    wgpu::ShaderStages::VERTEX,
                    wgpu::BufferBindingType::Storage { read_only: true },
                    true,
                ),
            );
            layouts.push(instances_layout);
        }

        let render_pipeline = self.caches.render_pipeline.get(
            self.wgpu,
            RenderPipelineConfig {
                shader: self.shader.clone(),
                vertex_buffer_layouts: &[MeshVertex::LAYOUT],
                blend_mode: self.blend_mode,
            },
            &layouts,
        );

        self.gpu_render_pass
            .set_pipeline(self.caches.ref_id_holder.hold(render_pipeline));
        self.dirty_pipline = false;
    }

    /// Set the rect area where pixels will be visible
    pub fn set_scissor_rect(&mut self, mut rect: Rect) {
        rect.constrain(self.target.size().as_vec2());
        self.gpu_render_pass.set_scissor_rect(
            rect.x as u32,
            rect.y as u32,
            rect.w as u32,
            rect.h as u32,
        );
    }

    pub fn reset_scissor_rect(&mut self) {
        let size = self.target.size();
        self.gpu_render_pass.set_scissor_rect(0, 0, size.x, size.y);
    }

    fn set_bind_group(&mut self, index: u32, bind_group: RefId<wgpu::BindGroup>, offsets: &[u32]) {
        self.gpu_render_pass.set_bind_group(
            index,
            // We need to use the holder that will hold the RefId until the end of the frame since
            // render_pipeline requires that and using RefId on it's own might drop the inner value
            self.caches.ref_id_holder.hold(bind_group),
            offsets,
        );
    }

    pub fn target_rect(&self) -> Rect {
        Rect {
            x: 0.,
            y: 0.,
            w: self.target.size().x as f32,
            h: self.target.size().y as f32,
        }
    }
}
