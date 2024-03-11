use std::num::NonZeroU64;

use super::{
    BindGroupBuilder, Color, GraphicsCaches, GraphicsContext, GraphicsData, MeshVertex, Rect,
    RenderPipelineConfig, Shader, ShaderKind, Texture, TextureView, WGPUContext,
};
use crate::{BindGroupLayoutBuilder, MeshView, RefId};

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
    has_depth: bool,
    dirty_pipline: bool,

    pub target_texture: TextureView<'a>,
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
        depth_texture: Option<&'a Texture>,
        target: Option<TextureView<'a>>,
    ) -> Self {
        let frame = gfx
            .frame
            .as_mut()
            .expect("tried to create render pass but frame doesn't exist");

        let target = target.unwrap_or(TextureView::new(&frame.output.texture, &frame.output_view));
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
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: depth_texture.map(|depth_texture| {
                    wgpu::RenderPassDepthStencilAttachment {
                        view: &depth_texture.gpu_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

        Self {
            gpu_render_pass,
            shader: gfx.data.single_draw_shader.clone(),
            dirty_pipline: true,
            target_texture: target,
            has_depth: depth_texture.is_some(),

            // We need to get references to WGPUContext, GraphicsData, GraphicsCaches seperately or rust is
            // going to complain about mutiple borrows
            wgpu: &gfx.wgpu,
            caches: &mut gfx.caches,
            data: &gfx.data,
        }
    }

    pub fn draw_mesh_instanced(&mut self, mesh: MeshView<'a>, instances: &[RenderInstance]) {
        let instances_alloc = self
            .caches
            .storage_buffer_allocator
            .alloc_write(self.wgpu, instances);

        let instances_bind_group = BindGroupBuilder::new()
            .buffer(
                wgpu::ShaderStages::VERTEX,
                &instances_alloc.buffer,
                wgpu::BufferBindingType::Storage { read_only: true },
                NonZeroU64::new(instances_alloc.size),
                true,
            )
            .build(self.wgpu);

        self.set_bind_group(2, instances_bind_group, &[instances_alloc.offset as u32]);
        self.draw_mesh(mesh, instances.len() as u32);
    }

    pub fn draw_mesh_single(&mut self, mesh: MeshView<'a>) {
        self.draw_mesh(mesh, 1);
    }

    fn draw_mesh(&mut self, mesh: MeshView<'a>, instance_count: u32) {
        self.update_pipeline();
        self.gpu_render_pass
            .set_index_buffer(mesh.index_buffer, wgpu::IndexFormat::Uint32);
        self.gpu_render_pass
            .set_vertex_buffer(0, mesh.vertex_buffer);
        self.gpu_render_pass
            .draw_indexed(0..mesh.index_count, 0, 0..instance_count);
    }

    pub fn set_texture(&mut self, texture: &RefId<Texture>) {
        let sampler = self.caches.sampler.get(self.wgpu, texture.sampler_config);
        let texture_bind_group = BindGroupBuilder::new()
            .texture(wgpu::ShaderStages::FRAGMENT, texture)
            .sampler(wgpu::ShaderStages::FRAGMENT, &sampler)
            .build(self.wgpu);

        self.set_bind_group(1, texture_bind_group, &[]);
    }

    pub fn set_uniform<U: bytemuck::Pod>(&mut self, uniform: U) {
        let uniform_alloc = self
            .caches
            .uniform_buffer_allocator
            .alloc_write(self.wgpu, &[uniform]);

        let uniform_bind_group = BindGroupBuilder::new()
            .buffer(
                wgpu::ShaderStages::VERTEX,
                &uniform_alloc.buffer,
                wgpu::BufferBindingType::Uniform,
                NonZeroU64::new(uniform_alloc.size),
                true,
            )
            .build(self.wgpu);

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

        let uniform_layout = self.caches.render_pipeline.get_bind_layout(
            self.wgpu,
            BindGroupLayoutBuilder::new().buffer(
                wgpu::ShaderStages::VERTEX,
                wgpu::BufferBindingType::Uniform,
                true,
            ),
        );

        let texture_layout = self.caches.render_pipeline.get_bind_layout(
            self.wgpu,
            BindGroupLayoutBuilder::new()
                .texture(wgpu::ShaderStages::FRAGMENT)
                .sampler(wgpu::ShaderStages::FRAGMENT),
        );

        let instances_layout = self.caches.render_pipeline.get_bind_layout(
            self.wgpu,
            BindGroupLayoutBuilder::new().buffer(
                wgpu::ShaderStages::VERTEX,
                wgpu::BufferBindingType::Storage { read_only: true },
                true,
            ),
        );

        let draw_single_layouts = [&uniform_layout, &texture_layout];
        let instanced_layouts = [&uniform_layout, &texture_layout, &instances_layout];
        let layouts = match self.shader.kind {
            ShaderKind::DrawSingle => draw_single_layouts.as_ref(),
            ShaderKind::Instanced => instanced_layouts.as_ref(),
        };

        let render_pipeline = self.caches.render_pipeline.get_pipeline(
            self.wgpu,
            RenderPipelineConfig {
                shader: self.shader.clone(),
                vertex_buffer_layouts: &[MeshVertex::LAYOUT],
                format: self.target_texture.gpu_texture.format(),
                has_depth: self.has_depth,
            },
            layouts,
        );

        self.gpu_render_pass.set_pipeline(render_pipeline);
        self.dirty_pipline = false;
    }

    /// Set the rect area where pixels will be visible
    pub fn set_scissor_rect(&mut self, mut rect: Rect) {
        rect.constrain(self.target_texture.size().as_vec2());
        self.gpu_render_pass.set_scissor_rect(
            rect.x as u32,
            rect.y as u32,
            rect.w as u32,
            rect.h as u32,
        );
    }

    pub fn reset_scissor_rect(&mut self) {
        let size = self.target_texture.size();
        self.gpu_render_pass.set_scissor_rect(0, 0, size.x, size.y);
    }

    fn set_bind_group(&mut self, index: u32, bind_group: wgpu::BindGroup, offsets: &[u32]) {
        // Bind group will get drop soon after it is created but it needs to live until the end of frame
        self.caches.bind_group_holder.push(bind_group);
        let bind_group = self.caches.bind_group_holder.last().unwrap();
        self.gpu_render_pass.set_bind_group(
            index,
            unsafe { &*(bind_group as *const wgpu::BindGroup) },
            offsets,
        );
    }

    pub fn target_rect(&self) -> Rect {
        Rect::from_pos_size(glam::Vec2::ZERO, self.target_texture.size().as_vec2())
    }
}
