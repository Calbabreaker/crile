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

#[repr(C)]
#[derive(Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
    pub transform: glam::Mat4,
    pub color: Color,
}

pub struct RenderPass<'a> {
    gpu_render_pass: wgpu::RenderPass<'a>,
    shader: RefId<wgpu::ShaderModule>,
    dirty_pipline: bool,

    wgpu: &'a WGPUContext,
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
            shader: gfx.data.single_draw_shader.clone(),
            dirty_pipline: true,

            // We need to get references to WGPUContext, GfxData, GfxCaches seperataly or rust is
            // going to complain about mutiple borrows
            wgpu: &gfx.wgpu,
            caches: &mut gfx.caches,
            data: &gfx.data,
        })
    }

    pub fn draw_mesh_instanced(&mut self, mesh: &'a Mesh, instances: &[Instance]) {
        let instances_size = (std::mem::size_of::<Instance>() * instances.len()) as u64;
        let instances_alloc = self
            .caches
            .storage_buffer_allocator
            .allocate(self.wgpu, instances_size);

        self.wgpu.queue.write_buffer(
            &instances_alloc.buffer,
            instances_alloc.offset,
            bytemuck::cast_slice(instances),
        );

        let instances_bind_group = self.caches.bind_group.get(
            self.wgpu,
            &BindGroupEntries::new().buffer(
                wgpu::ShaderStages::VERTEX,
                &instances_alloc.buffer,
                wgpu::BufferBindingType::Storage { read_only: true },
                NonZeroU64::new(instances_size),
                true,
            ),
        );

        self.gpu_render_pass.set_bind_group(
            2,
            &instances_bind_group,
            &[instances_alloc.offset as u32],
        );
        self.update_pipeline();
        self.gpu_render_pass
            .set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        self.gpu_render_pass
            .set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.gpu_render_pass
            .draw_indexed(0..mesh.index_count, 0, 0..instances.len() as u32);
    }

    pub fn draw_mesh_single(&mut self, mesh: &'a Mesh) {
        self.update_pipeline();
        self.gpu_render_pass
            .set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        self.gpu_render_pass
            .set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.gpu_render_pass
            .draw_indexed(0..mesh.index_count, 0, 0..1);
    }

    pub fn set_texture(&mut self, texture: &Texture) {
        let sampler = self.caches.sampler.get(self.wgpu, texture.sampler_config);
        let texture_bind_group = self.caches.bind_group.get(
            self.wgpu,
            &BindGroupEntries::new()
                .texture(wgpu::ShaderStages::FRAGMENT, &texture.gpu_view)
                .sampler(wgpu::ShaderStages::FRAGMENT, &sampler),
        );

        self.gpu_render_pass
            .set_bind_group(1, &texture_bind_group, &[]);
    }

    pub fn set_uniform<U: bytemuck::Pod>(&mut self, uniform: U) {
        let uniform_size = std::mem::size_of::<U>() as u64;
        let uniform_alloc = self
            .caches
            .uniform_buffer_allocator
            .allocate(self.wgpu, uniform_size);

        self.wgpu.queue.write_buffer(
            &uniform_alloc.buffer,
            uniform_alloc.offset,
            bytemuck::cast_slice(&[uniform]),
        );

        let uniform_bind_group = self.caches.bind_group.get(
            self.wgpu,
            &BindGroupEntries::new().buffer(
                wgpu::ShaderStages::VERTEX,
                &uniform_alloc.buffer,
                wgpu::BufferBindingType::Uniform,
                NonZeroU64::new(uniform_size),
                true,
            ),
        );

        self.gpu_render_pass
            .set_bind_group(0, &uniform_bind_group, &[uniform_alloc.offset as u32]);
    }

    pub fn set_shader(&mut self, shader: RefId<wgpu::ShaderModule>) {
        self.shader = shader;
        self.dirty_pipline = true;
    }

    pub fn update_pipeline(&mut self) {
        if self.dirty_pipline {
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

            let instances_layout = self.caches.bind_group.get_layout(
                self.wgpu,
                &BindGroupEntries::new().buffer_layout(
                    wgpu::ShaderStages::VERTEX,
                    wgpu::BufferBindingType::Storage { read_only: true },
                    true,
                ),
            );

            let render_pipeline = self.caches.render_pipeline.get(
                self.wgpu,
                RenderPipelineConfig {
                    shader: self.shader.clone(),
                    vertex_buffer_layouts: &[MeshVertex::LAYOUT],
                },
                &[&uniform_layout, &texture_layout, &instances_layout],
            );

            self.gpu_render_pass.set_pipeline(render_pipeline);
            self.dirty_pipline = false;
        }
    }
}
