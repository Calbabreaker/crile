use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use super::WgpuContext;
use crate::{BindGroupLayoutBuilder, NoHashHashMap, RefId, Texture};

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct RenderPipelineConfig {
    pub shader: RefId<Shader>,
    pub vertex_buffer_layouts: &'static [wgpu::VertexBufferLayout<'static>],
    pub format: wgpu::TextureFormat,
    pub has_depth: bool,
}

impl From<&RenderPipelineConfig> for RenderPipelineConfig {
    fn from(value: &RenderPipelineConfig) -> Self {
        value.clone()
    }
}

/// Caches gpu resources to prevent unnesscery creation
/// Everytime the bind groups, we need to recreate the pipeline
#[derive(Default)]
pub struct RenderPipelineCache {
    pipeline_cache: hashbrown::HashMap<RenderPipelineConfig, RefId<wgpu::RenderPipeline>>,
    layout_cache: NoHashHashMap<u64, RefId<wgpu::PipelineLayout>>,
    bind_layout_cache:
        hashbrown::HashMap<Vec<wgpu::BindGroupLayoutEntry>, RefId<wgpu::BindGroupLayout>>,
}

impl RenderPipelineCache {
    pub fn get_pipeline(
        &mut self,
        wgpu: &WgpuContext,
        config: RenderPipelineConfig,
        bind_group_layouts: &[&RefId<wgpu::BindGroupLayout>],
    ) -> RefId<wgpu::RenderPipeline> {
        let layout = self.get_layout(wgpu, bind_group_layouts);
        self.pipeline_cache
            .entry_ref(&config)
            .or_insert_with(|| RefId::new(create_pipeline(&wgpu.device, &layout, &config)))
            .clone()
    }

    fn get_layout(
        &mut self,
        wgpu: &WgpuContext,
        bind_group_layouts: &[&RefId<wgpu::BindGroupLayout>],
    ) -> RefId<wgpu::PipelineLayout> {
        // Calculate hash from bind_group_layouts ids
        let mut hasher = DefaultHasher::new();
        for layouts in bind_group_layouts {
            layouts.id().hash(&mut hasher);
        }
        let key = hasher.finish();

        self.layout_cache
            .entry(key)
            .or_insert_with(|| RefId::new(create_pipeline_layout(&wgpu.device, bind_group_layouts)))
            .clone()
    }

    pub fn get_bind_layout(
        &mut self,
        wgpu: &WgpuContext,
        builder: BindGroupLayoutBuilder,
    ) -> RefId<wgpu::BindGroupLayout> {
        self.bind_layout_cache
            .entry_ref(builder.entries())
            .or_insert_with(|| RefId::new(builder.build(wgpu)))
            .clone()
    }
}

fn create_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    config: &RenderPipelineConfig,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &config.shader.module,
            entry_point: Some("vs_main"),
            buffers: config.vertex_buffer_layouts,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &config.shader.module,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
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
        depth_stencil: if config.has_depth {
            Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            })
        } else {
            None
        },
        multiview: None,
        cache: None,
    })
}

fn create_pipeline_layout(
    device: &wgpu::Device,
    bind_group_layouts: &[&RefId<wgpu::BindGroupLayout>],
) -> wgpu::PipelineLayout {
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &bind_group_layouts
            .iter()
            .map(|b| b.as_ref())
            .collect::<Vec<_>>(),
        push_constant_ranges: &[],
    })
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
        wgpu: &WgpuContext,
        descriptor: wgpu::ShaderModuleDescriptor,
        kind: ShaderKind,
    ) -> Self {
        Self {
            module: wgpu.device.create_shader_module(descriptor),
            kind,
        }
    }
}
