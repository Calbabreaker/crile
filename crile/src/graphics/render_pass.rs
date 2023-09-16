use crate::{
    Color, EngineError, GraphicsContext, GraphicsContextData, Matrix4, Mesh, Texture, WGPUContext,
};

#[repr(C)]
#[derive(Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DrawUniform {
    pub transform: Matrix4,
}

pub struct RenderPass<'a> {
    wgpu: &'a WGPUContext,
    pub gfx_data: &'a GraphicsContextData,
    gpu_render_pass: wgpu::RenderPass<'a>,
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
            // We need to get references to WGPUContext and GraphicsContextData since GraphicsContext is being mutably borrowed already when begin_render_pass (can't access it)
            gfx_data: &gfx.data,
            wgpu: &gfx.wgpu,
            gpu_render_pass,
        })
    }

    pub fn draw_mesh_indexed(
        &mut self,
        mesh: &'a Mesh,
        texture: &'a Texture,
        uniform: DrawUniform,
    ) {
        let data = self.gfx_data;
        self.wgpu.queue.write_buffer(
            &data.draw_uniform_buffer,
            0,
            bytemuck::cast_slice(&[uniform]),
        );

        self.gpu_render_pass
            .set_pipeline(&data.render_pipeline.gpu_pipeline);
        self.gpu_render_pass
            .set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        self.gpu_render_pass
            .set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.gpu_render_pass
            .set_bind_group(0, &data.uniform_bind_group.gpu_group, &[]);
        self.gpu_render_pass
            .set_bind_group(1, &data.texture_bind_group.gpu_group, &[]);
        self.gpu_render_pass
            .draw_indexed(0..mesh.index_count, 0, 0..10);
    }
}
