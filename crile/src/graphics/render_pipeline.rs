use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use crate::{RefId, WGPUContext};

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct RenderPipelineConfig {
    pub shader: RefId<Shader>,
    pub vertex_buffer_layouts: &'static [wgpu::VertexBufferLayout<'static>],
    pub blend_mode: wgpu::BlendState,
}

/// Caches gpu resources to prevent unnesscery creation
/// Everytime the bind groups, we need to recreate the pipeline
#[derive(Default)]
pub struct RenderPipelineCache {
    pipeline_cache: hashbrown::HashMap<RenderPipelineConfig, RefId<wgpu::RenderPipeline>>,
    layout_cache: hashbrown::HashMap<u64, RefId<wgpu::PipelineLayout>>,
}

impl RenderPipelineCache {
    pub fn get(
        &mut self,
        wgpu: &WGPUContext,
        config: RenderPipelineConfig,
        bind_group_layouts: &[RefId<wgpu::BindGroupLayout>],
    ) -> RefId<wgpu::RenderPipeline> {
        let layout = self.get_layout(wgpu, bind_group_layouts);
        let pipeline = self
            .pipeline_cache
            .entry(config.clone())
            .or_insert_with(|| {
                wgpu.device
                    .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: None,
                        layout: Some(&layout),
                        vertex: wgpu::VertexState {
                            module: &config.shader.module,
                            entry_point: "vs_main",
                            buffers: config.vertex_buffer_layouts,
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: &config.shader.module,
                            entry_point: "fs_main",
                            targets: &[Some(wgpu::ColorTargetState {
                                format: wgpu.surface_config.format,
                                blend: Some(config.blend_mode),
                                write_mask: wgpu::ColorWrites::ALL,
                            })],
                        }),
                        primitive: wgpu::PrimitiveState {
                            topology: wgpu::PrimitiveTopology::TriangleList,
                            strip_index_format: None,
                            front_face: wgpu::FrontFace::Ccw,
                            cull_mode: None,
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
            });

        RefId::clone(pipeline)
    }

    fn get_layout(
        &mut self,
        wgpu: &WGPUContext,
        bind_group_layouts: &[RefId<wgpu::BindGroupLayout>],
    ) -> RefId<wgpu::PipelineLayout> {
        // Calculate hash from bind_group_layouts ids
        let mut hasher = DefaultHasher::new();
        for bind_group in bind_group_layouts {
            bind_group.id().hash(&mut hasher);
        }
        let key = hasher.finish();

        let layout = self.layout_cache.entry(key).or_insert_with(|| {
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
        });
        RefId::clone(layout)
    }
}

#[derive(PartialEq, Eq)]
pub enum ShaderKind {
    DrawSingle,
    Instanced,
}

pub struct Shader {
    pub module: wgpu::ShaderModule,
    pub kind: ShaderKind,
}

impl Shader {
    pub fn new(
        wgpu: &WGPUContext,
        descriptor: wgpu::ShaderModuleDescriptor,
        kind: ShaderKind,
    ) -> Self {
        Self {
            module: wgpu.device.create_shader_module(descriptor),
            kind,
        }
    }
}
