use crate::{BindGroup, RendererAPI};

pub struct RenderPipelineConfig<'a> {
    pub shader: wgpu::ShaderModuleDescriptor<'a>,
    pub bind_groups: Vec<BindGroup>,
    pub vertex_buffer_layouts: &'a [wgpu::VertexBufferLayout<'a>],
}

pub struct RenderPipeline {
    pub gpu_pipeline: wgpu::RenderPipeline,
    pub bind_groups: Vec<BindGroup>,
}

impl RenderPipeline {
    pub fn new(api: &RendererAPI, config: RenderPipelineConfig) -> Self {
        let gpu_pipeline_layout =
            api.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &config
                        .bind_groups
                        .iter()
                        .map(|bind_group| &bind_group.gpu_layout)
                        .collect::<Vec<_>>(),
                    push_constant_ranges: &[],
                });

        let gpu_shader = api.device.create_shader_module(config.shader);

        let gpu_pipeline = api
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&gpu_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &gpu_shader,
                    entry_point: "vs_main",
                    buffers: &config.vertex_buffer_layouts,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &gpu_shader,
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
            gpu_pipeline,
            bind_groups: config.bind_groups,
        }
    }

    pub fn bind<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.gpu_pipeline);
        for (i, bind_group) in self.bind_groups.iter().enumerate() {
            render_pass.set_bind_group(i as u32, &bind_group.gpu_group, &[]);
        }
    }
}
