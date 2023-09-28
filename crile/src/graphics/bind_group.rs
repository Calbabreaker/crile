use std::num::NonZeroU64;

use crate::{RefId, WGPUContext};

/// Key to differentiate between bind groups in cache
/// Not every property is used, only the ones that change
#[derive(Hash, PartialEq, Eq, Clone)]
enum BindGroupEntryKey {
    Buffer { id: u64, size: Option<NonZeroU64> },
    Texture { id: u64 },
    Sampler { id: u64 },
}

/// Builder-like pattern to create bind groups
#[derive(Default)]
pub struct BindGroupEntries<'a> {
    keys: Vec<BindGroupEntryKey>,
    layouts: Vec<wgpu::BindGroupLayoutEntry>,
    groups: Vec<wgpu::BindGroupEntry<'a>>,
}

impl<'a> BindGroupEntries<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn buffer(
        mut self,
        visibility: wgpu::ShaderStages,
        buffer: &'a RefId<wgpu::Buffer>,
        ty: wgpu::BufferBindingType,
        size: Option<NonZeroU64>,
        has_dynamic_offset: bool,
    ) -> Self {
        self.keys.push(BindGroupEntryKey::Buffer {
            id: buffer.id(),
            size,
        });
        self.groups.push(wgpu::BindGroupEntry {
            binding: self.groups.len() as u32,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer,
                offset: 0,
                size,
            }),
        });
        self.buffer_layout(visibility, ty, has_dynamic_offset)
    }

    pub fn texture(
        mut self,
        visibility: wgpu::ShaderStages,
        view: &'a RefId<wgpu::TextureView>,
    ) -> Self {
        self.keys.push(BindGroupEntryKey::Texture { id: view.id() });
        self.groups.push(wgpu::BindGroupEntry {
            binding: self.groups.len() as u32,
            resource: wgpu::BindingResource::TextureView(view),
        });
        self.texture_layout(visibility)
    }

    pub fn sampler(
        mut self,
        visibility: wgpu::ShaderStages,
        sampler: &'a RefId<wgpu::Sampler>,
    ) -> Self {
        self.keys
            .push(BindGroupEntryKey::Sampler { id: sampler.id() });
        self.groups.push(wgpu::BindGroupEntry {
            binding: self.groups.len() as u32,
            resource: wgpu::BindingResource::Sampler(sampler),
        });
        self.sampler_layout(visibility)
    }

    // Layout only versions of the functions above (to be used when creating the pipeline)
    pub fn buffer_layout(
        mut self,
        visibility: wgpu::ShaderStages,
        ty: wgpu::BufferBindingType,
        has_dynamic_offset: bool,
    ) -> Self {
        self.layouts.push(wgpu::BindGroupLayoutEntry {
            binding: self.layouts.len() as u32,
            ty: wgpu::BindingType::Buffer {
                ty,
                has_dynamic_offset,
                min_binding_size: None,
            },
            visibility,
            count: None,
        });
        self
    }

    pub fn texture_layout(mut self, visibility: wgpu::ShaderStages) -> Self {
        self.layouts.push(wgpu::BindGroupLayoutEntry {
            binding: self.layouts.len() as u32,
            visibility,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
            },
            count: None,
        });
        self
    }

    pub fn sampler_layout(mut self, visibility: wgpu::ShaderStages) -> Self {
        self.layouts.push(wgpu::BindGroupLayoutEntry {
            binding: self.layouts.len() as u32,
            visibility,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        });
        self
    }
}

/// Everytime we need to use a different buffer or texture, or the layout buffer or texture changes, we need to recreate the bind group
/// This cache allows for that only when necessery
/// TODO: figure out when to free items from the caches
#[derive(Default)]
pub struct BindGroupCache {
    // The layout sometimes doesn't have to change if the group does
    group_cache: hashbrown::HashMap<Vec<BindGroupEntryKey>, RefId<wgpu::BindGroup>>,
    layout_cache: hashbrown::HashMap<Vec<wgpu::BindGroupLayoutEntry>, RefId<wgpu::BindGroupLayout>>,
}

impl BindGroupCache {
    pub fn get(
        &mut self,
        wgpu: &WGPUContext,
        entries: &BindGroupEntries,
    ) -> RefId<wgpu::BindGroup> {
        let layout = self.get_layout(wgpu, entries);
        let group = self
            .group_cache
            .entry_ref(entries.keys.as_slice())
            .or_insert_with(|| {
                wgpu.device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        label: None,
                        layout: layout.as_ref(),
                        entries: &entries.groups,
                    })
                    .into()
            });

        RefId::clone(group)
    }

    pub fn get_layout(
        &mut self,
        wgpu: &WGPUContext,
        entries: &BindGroupEntries,
    ) -> RefId<wgpu::BindGroupLayout> {
        let layout = self
            .layout_cache
            .entry_ref(entries.layouts.as_slice())
            .or_insert_with(|| {
                wgpu.device
                    .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        label: None,
                        entries: &entries.layouts,
                    })
                    .into()
            });

        RefId::clone(layout)
    }
}
