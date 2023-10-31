use std::{mem::MaybeUninit, num::NonZeroU64};

use super::WGPUContext;
use crate::{RefId, Texture};

pub struct BindGroupLayoutBuilder<const SIZE: usize> {
    entries: [wgpu::BindGroupLayoutEntry; SIZE],
    length: usize,
}

impl<const SIZE: usize> BindGroupLayoutBuilder<SIZE> {
    pub fn new() -> Self {
        unsafe {
            Self {
                // Need to start uninitialized since these types don't impl Default
                // Should be safe since we ensure length and size matches on bind group creation
                entries: [MaybeUninit::zeroed().assume_init(); SIZE],
                length: 0,
            }
        }
    }

    pub fn buffer(
        mut self,
        visibility: wgpu::ShaderStages,
        ty: wgpu::BufferBindingType,
        has_dynamic_offset: bool,
    ) -> Self {
        self.entries[self.length] = wgpu::BindGroupLayoutEntry {
            binding: self.length as u32,
            ty: wgpu::BindingType::Buffer {
                ty,
                has_dynamic_offset,
                min_binding_size: None,
            },
            visibility,
            count: None,
        };
        self.length += 1;
        self
    }

    pub fn texture(mut self, visibility: wgpu::ShaderStages) -> Self {
        self.entries[self.length] = wgpu::BindGroupLayoutEntry {
            binding: self.length as u32,
            visibility,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
            },
            count: None,
        };
        self.length += 1;
        self
    }

    pub fn sampler(mut self, visibility: wgpu::ShaderStages) -> Self {
        self.entries[self.length] = wgpu::BindGroupLayoutEntry {
            binding: self.length as u32,
            visibility,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        };
        self.length += 1;
        self
    }

    pub fn build(&self, wgpu: &WGPUContext) -> wgpu::BindGroupLayout {
        wgpu.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: self.entries(),
            })
    }

    pub fn entries(&self) -> &[wgpu::BindGroupLayoutEntry] {
        assert_eq!(self.length, SIZE);
        &self.entries
    }
}

impl<const SIZE: usize> Default for BindGroupLayoutBuilder<SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder-like pattern to create bind groups
pub struct BindGroupBuilder<'a, const SIZE: usize> {
    layout_builder: BindGroupLayoutBuilder<SIZE>,
    entries: [wgpu::BindGroupEntry<'a>; SIZE],
}

impl<'a, const SIZE: usize> BindGroupBuilder<'a, SIZE> {
    pub fn new() -> Self {
        unsafe {
            Self {
                layout_builder: BindGroupLayoutBuilder::new(),
                entries: std::array::from_fn(|_| MaybeUninit::zeroed().assume_init()),
            }
        }
    }

    pub fn buffer(
        mut self,
        visibility: wgpu::ShaderStages,
        buffer: &'a RefId<wgpu::Buffer>,
        ty: wgpu::BufferBindingType,
        size: Option<NonZeroU64>,
        has_dynamic_offset: bool,
    ) -> Self {
        self.entries[self.layout_builder.length] = wgpu::BindGroupEntry {
            binding: self.layout_builder.length as u32,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer,
                offset: 0,
                size,
            }),
        };
        self.layout_builder = self
            .layout_builder
            .buffer(visibility, ty, has_dynamic_offset);
        self
    }

    pub fn texture(mut self, visibility: wgpu::ShaderStages, texture: &'a RefId<Texture>) -> Self {
        self.entries[self.layout_builder.length] = wgpu::BindGroupEntry {
            binding: self.layout_builder.length as u32,
            resource: wgpu::BindingResource::TextureView(&texture.gpu_view),
        };
        self.layout_builder = self.layout_builder.texture(visibility);
        self
    }

    pub fn sampler(
        mut self,
        visibility: wgpu::ShaderStages,
        sampler: &'a RefId<wgpu::Sampler>,
    ) -> Self {
        self.entries[self.layout_builder.length] = wgpu::BindGroupEntry {
            binding: self.layout_builder.length as u32,
            resource: wgpu::BindingResource::Sampler(sampler),
        };
        self.layout_builder = self.layout_builder.sampler(visibility);
        self
    }

    pub fn build(&mut self, wgpu: &WGPUContext) -> wgpu::BindGroup {
        wgpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.layout_builder.build(wgpu),
            entries: &self.entries,
        })
    }
}

impl<'a, const SIZE: usize> Default for BindGroupBuilder<'a, SIZE> {
    fn default() -> Self {
        Self::new()
    }
}
