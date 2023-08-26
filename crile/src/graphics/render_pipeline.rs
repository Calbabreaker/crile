use crate::{BindGroup, Buffer, RenderInstance, RendererAPI};

pub struct RenderPipelineConfig<'a> {
    pub shader: wgpu::ShaderModuleDescriptor<'a>,
    pub bind_groups: Vec<BindGroup>,
    pub vertex_buffers: Vec<Buffer>,
    pub index_buffer: Buffer,
}

pub struct RenderPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_groups: Vec<BindGroup>,
    pub vertex_buffers: Vec<Buffer>,
    pub index_buffer: Buffer,
}

impl RenderPipeline {
    pub fn new(api: &RendererAPI, config: RenderPipelineConfig) -> Self {
        let render_pipeline_layout =
            api.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &config
                        .bind_groups
                        .iter()
                        .map(|bind_group| &bind_group.layout)
                        .collect::<Vec<_>>(),
                    push_constant_ranges: &[],
                });

        let shader = api.device.create_shader_module(config.shader);

        let pipeline = api
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &config
                        .vertex_buffers
                        .iter()
                        .map(|buffer| {
                            buffer
                                .layout
                                .clone()
                                .expect("vertex buffer should have layout")
                        })
                        .collect::<Vec<_>>(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: api.config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                depth_stencil: None,
                multiview: None,
            });

        Self {
            pipeline,
            bind_groups: config.bind_groups,
            index_buffer: config.index_buffer,
            vertex_buffers: config.vertex_buffers,
        }
    }

    pub fn draw_indexed<T: 'static>(&self, render_instance: &mut RenderInstance, indices: &[T]) {
        let mut render_pass = render_instance.begin_render_pass();
        render_pass.set_pipeline(&self.pipeline);
        for (i, bind_group) in self.bind_groups.iter().enumerate() {
            render_pass.set_bind_group(i as u32, &bind_group.group, &[]);
        }
        for (i, vertex_buffer) in self.vertex_buffers.iter().enumerate() {
            render_pass.set_vertex_buffer(i as u32, vertex_buffer.buffer.slice(..));
        }
        render_pass.set_index_buffer(self.index_buffer.buffer.slice(..), get_index_format::<T>());
        render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
    }
}

fn get_index_format<T: 'static>() -> wgpu::IndexFormat {
    use std::any::TypeId;
    if TypeId::of::<T>() == TypeId::of::<u32>() {
        wgpu::IndexFormat::Uint32
    } else if TypeId::of::<T>() == TypeId::of::<u16>() {
        wgpu::IndexFormat::Uint16
    } else {
        panic!("Only u16 or u32 expected");
    }
}
