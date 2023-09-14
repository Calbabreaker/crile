use crate::RendererAPI;

pub struct Texture {
    pub gpu_texture: wgpu::Texture,
    pub gpu_view: wgpu::TextureView,
    pub gpu_sampler: wgpu::Sampler,
}

impl Texture {
    pub fn from_image(api: &RendererAPI, image: image::DynamicImage) -> Self {
        Self::new(api, image.width(), image.height(), &image.to_rgba8())
    }

    /// Creats a new texture to be rendrered
    /// Note: only expects rgba8 images
    pub fn new(api: &RendererAPI, width: u32, height: u32, data: &[u8]) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let gpu_texture = api.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        api.queue.write_texture(
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
        let gpu_sampler = api.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            gpu_texture,
            gpu_view,
            gpu_sampler,
        }
    }
}
