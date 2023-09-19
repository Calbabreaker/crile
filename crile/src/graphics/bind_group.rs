// stdlib hashmap does not have entry_ref
use hashbrown::HashMap;
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
        self.groups.push(wgpu::BindGroupEntry {
            binding: self.groups.len() as u32,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer,
                offset: 0,
                size,
            }),
        });
        self
    }

    pub fn texture(
        mut self,
        visibility: wgpu::ShaderStages,
        view: &'a RefId<wgpu::TextureView>,
    ) -> Self {
        self.keys.push(BindGroupEntryKey::Texture { id: view.id() });
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
        self.groups.push(wgpu::BindGroupEntry {
            binding: self.groups.len() as u32,
            resource: wgpu::BindingResource::TextureView(view),
        });
        self
    }

    pub fn sampler(
        mut self,
        visibility: wgpu::ShaderStages,
        sampler: &'a RefId<wgpu::Sampler>,
    ) -> Self {
        self.keys
            .push(BindGroupEntryKey::Sampler { id: sampler.id() });
        self.layouts.push(wgpu::BindGroupLayoutEntry {
            binding: self.layouts.len() as u32,
            visibility,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        });
        self.groups.push(wgpu::BindGroupEntry {
            binding: self.groups.len() as u32,
            resource: wgpu::BindingResource::Sampler(sampler),
        });
        self
    }
}

/// Everytime we need to use a different buffer or texture, or the layout buffer or texture changes, we need to recreate the bind group
/// This cache allows for that only when necessery
#[derive(Default)]
pub struct BindGroupCache {
    // The layout sometimes doesn't have to change if the group does
    group_cache: HashMap<Vec<BindGroupEntryKey>, RefId<wgpu::BindGroup>>,
    layout_cache: HashMap<Vec<wgpu::BindGroupLayoutEntry>, RefId<wgpu::BindGroupLayout>>,
}

impl BindGroupCache {
    pub fn get(
        &mut self,
        wgpu: &WGPUContext,
        entries: &BindGroupEntries,
    ) -> (
        &'static RefId<wgpu::BindGroup>,
        &'static RefId<wgpu::BindGroupLayout>,
    ) {
        let layout = &self.get_layout(wgpu, &entries.layouts);
        let group = self
            .group_cache
            .entry_ref(entries.keys.as_slice())
            .or_insert_with(|| {
                wgpu.device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        label: None,
                        layout,
                        entries: &entries.groups,
                    })
                    .into()
            });

        // SAFETY: bind groups and render pipelines caches return a RefId<T>.
        // We can't use RefId<T> by itself since they will be dropped at the end of this function.
        // std::mem::transmute needs to be used to convert to a 'static RefId<T> which is unsafe
        // This requires the caches to not delete anything or be deleted while a frame is in progress to be safe
        unsafe { (std::mem::transmute(group), layout) }
    }

    pub fn get_layout(
        &mut self,
        wgpu: &WGPUContext,
        entries: &[wgpu::BindGroupLayoutEntry],
    ) -> &'static RefId<wgpu::BindGroupLayout> {
        let layout = self.layout_cache.entry_ref(entries).or_insert_with(|| {
            wgpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &entries,
                })
                .into()
        });
        unsafe { std::mem::transmute(layout) }
    }
}
