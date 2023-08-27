use crate::{Buffer, RendererAPI, Texture};

pub struct BindGroupEntry<'a> {
    pub ty: wgpu::BindingType,
    pub resource: wgpu::BindingResource<'a>,
    pub visibility: wgpu::ShaderStages,
}

impl<'a> BindGroupEntry<'a> {
    pub fn from_buffer<T>(visibility: wgpu::ShaderStages, buffer: &'a Buffer<T>) -> Self {
        Self {
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            visibility,
            resource: buffer.gpu_buffer.as_entire_binding(),
        }
    }

    pub fn from_texture(texture: &'a Texture) -> Vec<Self> {
        vec![
            Self {
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                resource: wgpu::BindingResource::TextureView(&texture.gpu_view),
                visibility: wgpu::ShaderStages::FRAGMENT,
            },
            Self {
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                visibility: wgpu::ShaderStages::FRAGMENT,
                resource: wgpu::BindingResource::Sampler(&texture.gpu_sampler),
            },
        ]
    }
}

pub struct BindGroup {
    pub gpu_group: wgpu::BindGroup,
    pub gpu_layout: wgpu::BindGroupLayout,
}

impl BindGroup {
    pub fn new(api: &RendererAPI, entries: &[BindGroupEntry]) -> Self {
        let gpu_layout = api
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &entries
                    .iter()
                    .enumerate()
                    .map(|(i, entry)| wgpu::BindGroupLayoutEntry {
                        binding: i as u32,
                        visibility: entry.visibility,
                        ty: entry.ty,
                        count: None,
                    })
                    .collect::<Vec<_>>(),
            });

        let gpu_group = api.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &gpu_layout,
            entries: &entries
                .iter()
                .enumerate()
                .map(|(i, entry)| wgpu::BindGroupEntry {
                    binding: i as u32,
                    resource: entry.resource.clone(),
                })
                .collect::<Vec<_>>(),
        });

        Self {
            gpu_layout,
            gpu_group,
        }
    }
}
