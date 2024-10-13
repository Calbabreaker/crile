use std::{mem::MaybeUninit, num::NonZeroU64};

use super::WgpuContext;
use crate::{RefId, Texture};

const MAX_SIZE: usize = 4;

pub struct BindGroupLayoutBuilder {
    entries: [wgpu::BindGroupLayoutEntry; MAX_SIZE],
    length: usize,
}

impl BindGroupLayoutBuilder {
    pub fn new() -> Self {
        unsafe {
            Self {
                // Need to use uninitialized array as we're using a stack allocated array and the length needs to vary
                // Should be safe since we only get entries up to self.length
                entries: [MaybeUninit::zeroed().assume_init(); MAX_SIZE],
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

    pub fn build(&self, wgpu: &WgpuContext) -> wgpu::BindGroupLayout {
        wgpu.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: self.entries(),
            })
    }

    pub fn entries(&self) -> &[wgpu::BindGroupLayoutEntry] {
        &self.entries[0..self.length]
    }
}

impl Default for BindGroupLayoutBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder-like pattern to create bind groups
pub struct BindGroupBuilder<'a> {
    layout_builder: BindGroupLayoutBuilder,
    entries: [wgpu::BindGroupEntry<'a>; MAX_SIZE],
}

impl<'a> BindGroupBuilder<'a> {
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

    pub fn build(&mut self, wgpu: &WgpuContext) -> wgpu::BindGroup {
        wgpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.layout_builder.build(wgpu),
            entries: self.entries(),
        })
    }

    pub fn entries(&self) -> &[wgpu::BindGroupEntry] {
        &self.entries[0..self.layout_builder.length]
    }
}

impl Default for BindGroupBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}
