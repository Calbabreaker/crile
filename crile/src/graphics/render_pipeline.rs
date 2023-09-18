use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
};

use crate::{ArcId, WGPUContext};

#[derive(Hash, PartialEq, Eq)]
pub struct RenderPipelineConfig<'a> {
    pub shader: ArcId<wgpu::ShaderModule>,
    pub vertex_buffer_layouts: &'a [wgpu::VertexBufferLayout<'a>],
    pub bind_group_layouts: &'a [ArcId<wgpu::BindGroupLayout>],
}

/// Caches gpu resources to prevent unnesscery creation
/// Everytime the bind groups, we need to recreate the pipeline
#[derive(Default)]
pub struct RenderPipelineCache {
    pipeline_cache: HashMap<RenderPipelineConfig<'static>, ArcId<wgpu::RenderPipeline>>,
    layout_cache: HashMap<u64, ArcId<wgpu::PipelineLayout>>,
}

impl RenderPipelineCache {
    pub fn get(
        &mut self,
        wgpu: &WGPUContext,
        config: RenderPipelineConfig,
    ) -> &ArcId<wgpu::RenderPipeline> {
        self.pipeline_cache.entry(config).or_insert_with(|| {
            wgpu.device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: None,
                    layout: Some(self.layout(wgpu, config.bind_group_layouts)),
                    vertex: wgpu::VertexState {
                        module: &config.shader,
                        entry_point: "vs_main",
                        buffers: config.vertex_buffer_layouts,
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &config.shader,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: wgpu.surface_config.format,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
                })
                .into()
        })
    }

    fn layout(
        &mut self,
        wgpu: &WGPUContext,
        bind_group_layouts: &[ArcId<wgpu::BindGroupLayout>],
    ) -> &ArcId<wgpu::PipelineLayout> {
        // Calculate hash from bind_group_layouts ids
        let mut hasher = DefaultHasher::new();
        for bind_group in bind_group_layouts {
            bind_group.id().hash(&mut hasher);
        }
        let key = hasher.finish();

        self.layout_cache.entry(key).or_insert_with(|| {
            wgpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &bind_group_layouts
                        .iter()
                        .map(|b| b.as_ref())
                        .collect::<Vec<_>>(),
                    push_constant_ranges: &[],
                })
                .into()
        })
    }
}
