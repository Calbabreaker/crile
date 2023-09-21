use crate::{RefId, WGPUContext};

/// Wrapper around the wgpu texture objects
/// cheap to clone
#[derive(Clone)]
pub struct Texture {
    pub gpu_texture: RefId<wgpu::Texture>,
    pub gpu_view: RefId<wgpu::TextureView>,
    pub sampler_config: SamplerConfig,
}

impl Texture {
    pub fn from_image(wgpu: &WGPUContext, image: image::DynamicImage) -> Self {
        Self::new(wgpu, image.width(), image.height(), &image.to_rgba8())
    }

    /// Creats a new texture to be rendrered
    /// Note: only expects rgba8 images
    pub fn new(wgpu: &WGPUContext, width: u32, height: u32, data: &[u8]) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let gpu_texture = wgpu.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        wgpu.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &gpu_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        let gpu_view = gpu_texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            gpu_texture: gpu_texture.into(),
            gpu_view: gpu_view.into(),
            sampler_config: SamplerConfig::linear(),
        }
    }

    pub fn as_ref(&self) -> TextureRef {
        TextureRef::new(&self.gpu_texture, &self.gpu_view)
    }
}

pub struct TextureRef<'a> {
    pub gpu_texture: &'a wgpu::Texture,
    pub gpu_view: &'a wgpu::TextureView,
}

impl<'a> TextureRef<'a> {
    pub fn new(gpu_texture: &'a wgpu::Texture, gpu_view: &'a wgpu::TextureView) -> Self {
        Self {
            gpu_texture,
            gpu_view,
        }
    }

    pub fn size(&self) -> glam::UVec2 {
        glam::uvec2(self.gpu_texture.width(), self.gpu_texture.height())
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct SamplerConfig {
    pub clamp_u: wgpu::AddressMode,
    pub clamp_v: wgpu::AddressMode,
    pub mag: wgpu::FilterMode,
    pub min: wgpu::FilterMode,
}

impl SamplerConfig {
    /// Creates a sampler with bilinear interpolation
    pub fn linear() -> Self {
        Self {
            clamp_u: wgpu::AddressMode::ClampToEdge,
            clamp_v: wgpu::AddressMode::ClampToEdge,
            mag: wgpu::FilterMode::Linear,
            min: wgpu::FilterMode::Linear,
        }
    }

    /// Creates a sampler with nearest neighbour interpolation
    pub fn nearest() -> Self {
        Self {
            clamp_u: wgpu::AddressMode::ClampToEdge,
            clamp_v: wgpu::AddressMode::ClampToEdge,
            mag: wgpu::FilterMode::Nearest,
            min: wgpu::FilterMode::Nearest,
        }
    }
}

#[derive(Default)]
pub struct SamplerCache {
    sampler_cache: hashbrown::HashMap<SamplerConfig, RefId<wgpu::Sampler>>,
}

impl SamplerCache {
    pub fn get(&mut self, wgpu: &WGPUContext, config: SamplerConfig) -> RefId<wgpu::Sampler> {
        let sampler = self.sampler_cache.entry(config).or_insert_with(|| {
            RefId::new(wgpu.device.create_sampler(&wgpu::SamplerDescriptor {
                label: None,
                address_mode_u: config.clamp_u,
                address_mode_v: config.clamp_v,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: config.mag,
                min_filter: config.min,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            }))
        });

        RefId::clone(sampler)
    }
}
