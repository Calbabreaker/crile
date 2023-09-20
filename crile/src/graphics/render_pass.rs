use std::num::NonZeroU64;

use crate::{
    BindGroupEntries, Color, EngineError, GfxCaches, GfxData, GraphicsContext, Mesh, MeshVertex,
    RefId, RenderPipelineConfig, Texture, WGPUContext,
};

#[repr(C)]
#[derive(Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DrawUniform {
    pub transform: glam::Mat4,
}

pub struct DrawMeshParams<'a, U: bytemuck::Pod> {
    pub mesh: &'a Mesh,
    pub texture: &'a Texture,
    pub uniform: U,
    pub shader: RefId<wgpu::ShaderModule>,
}

pub struct RenderPass<'a> {
    wgpu: &'a WGPUContext,
    gpu_render_pass: wgpu::RenderPass<'a>,

    caches: &'a mut GfxCaches,
    pub data: &'a GfxData,
}

impl<'a> RenderPass<'a> {
    pub fn new(
        gfx: &'a mut GraphicsContext,
        clear_color: Option<Color>,
    ) -> Result<Self, EngineError> {
        let frame = gfx.frame.as_mut().ok_or(EngineError::RenderError(
            "tried to create render pass but frame doesn't exist".to_string(),
        ))?;

        let gpu_render_pass = frame
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &frame.output_view,
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

        Ok(Self {
            gpu_render_pass,

            // We need to get references to WGPUContext, GfxData, GfxCaches seperataly or rust is
            // going to complain about mutiple borrows
            wgpu: &gfx.wgpu,
            caches: &mut gfx.caches,
            data: &gfx.data,
        })
    }

    pub fn draw_mesh_indexed<U: bytemuck::Pod>(&mut self, params: DrawMeshParams<'a, U>) {
        let uniform_size = std::mem::size_of::<U>() as u64;
        let uniform_alloc = self
            .caches
            .buffer_allocator
            .allocate(self.wgpu, uniform_size);

        self.wgpu.queue.write_buffer(
            &uniform_alloc.buffer,
            uniform_alloc.offset,
            bytemuck::cast_slice(&[params.uniform]),
        );

        let (uniform_bind_group, uniform_bind_group_layout) = self.caches.bind_group.get(
            self.wgpu,
            &BindGroupEntries::new().buffer(
                wgpu::ShaderStages::VERTEX,
                &uniform_alloc.buffer,
                wgpu::BufferBindingType::Uniform,
                NonZeroU64::new(uniform_size),
                true,
            ),
        );

        let sampler = self
            .caches
            .sampler
            .get(self.wgpu, params.texture.sampler_config);
        let (texture_bind_group, texture_bind_group_layout) = self.caches.bind_group.get(
            self.wgpu,
            &BindGroupEntries::new()
                .texture(wgpu::ShaderStages::FRAGMENT, &params.texture.gpu_view)
                .sampler(wgpu::ShaderStages::FRAGMENT, &sampler),
        );

        let render_pipeline = self.caches.render_pipeline.get(
            self.wgpu,
            RenderPipelineConfig {
                shader: RefId::clone(&params.shader),
                vertex_buffer_layouts: &[MeshVertex::LAYOUT],
            },
            &[&uniform_bind_group_layout, &texture_bind_group_layout],
        );

        self.gpu_render_pass.set_pipeline(&render_pipeline);
        self.gpu_render_pass.set_index_buffer(
            params.mesh.index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );
        self.gpu_render_pass
            .set_vertex_buffer(0, params.mesh.vertex_buffer.slice(..));
        self.gpu_render_pass
            .set_bind_group(0, &uniform_bind_group, &[uniform_alloc.offset as u32]);
        self.gpu_render_pass
            .set_bind_group(1, &texture_bind_group, &[]);
        self.gpu_render_pass
            .draw_indexed(0..params.mesh.index_count, 0, 0..10);
    }
}
