use super::{
    DynamicBufferAllocator, Mesh, RenderPipelineCache, SamplerCache, Shader, ShaderKind, Texture,
};
use crate::{RefId, Window};

pub struct GraphicsContext {
    pub wgpu: WGPUContext,
    pub frame: Option<FrameContext>,
    pub data: GraphicsData,
    pub caches: GraphicsCaches,
}

impl GraphicsContext {
    pub fn new(window: &Window) -> Self {
        let wgpu = pollster::block_on(WGPUContext::new(window));

        let single_draw_shader = Shader::new(
            &wgpu,
            wgpu::include_wgsl!("./shaders/single_draw.wgsl"),
            ShaderKind::DrawSingle,
        );

        let instanced_shader = Shader::new(
            &wgpu,
            wgpu::include_wgsl!("./shaders/instanced.wgsl"),
            ShaderKind::Instanced,
        );

        let white_texture = Texture::from_pixels(&wgpu, 1, 1, &[255, 255, 255, 255]);
        let uniform_buffer_allocator =
            DynamicBufferAllocator::new(&wgpu, wgpu::BufferUsages::UNIFORM);
        let storage_buffer_allocator =
            DynamicBufferAllocator::new(&wgpu, wgpu::BufferUsages::STORAGE);
        let vertex_buffer_allocator =
            DynamicBufferAllocator::new(&wgpu, wgpu::BufferUsages::VERTEX);
        let index_buffer_allocator = DynamicBufferAllocator::new(&wgpu, wgpu::BufferUsages::INDEX);

        Self {
            data: GraphicsData {
                square_mesh: Mesh::new_square(&wgpu),
                white_texture: RefId::new(white_texture),
                instanced_shader: RefId::new(instanced_shader),
                single_draw_shader: RefId::new(single_draw_shader),
            },
            caches: GraphicsCaches {
                bind_group_holder: Vec::new(),
                render_pipeline: RenderPipelineCache::default(),
                sampler: SamplerCache::default(),
                uniform_buffer_allocator,
                storage_buffer_allocator,
                vertex_buffer_allocator,
                index_buffer_allocator,
            },
            wgpu,
            frame: None,
        }
    }

    pub fn resize(&mut self, size: glam::UVec2) {
        let wgpu = &mut self.wgpu;
        wgpu.surface_config.width = size.x;
        wgpu.surface_config.height = size.y;
        wgpu.surface.configure(&wgpu.device, &wgpu.surface_config);
    }

    /// Tries to enable/disable vsync
    /// On some platforms disabling vsync is not possible
    pub fn set_vsync(&mut self, enable: bool) {
        let wgpu = &mut self.wgpu;
        wgpu.surface_config.present_mode = match enable {
            true => wgpu::PresentMode::AutoVsync,
            false => wgpu::PresentMode::AutoNoVsync,
        };
        wgpu.surface.configure(&wgpu.device, &wgpu.surface_config);
    }

    pub fn vsync_enabled(&self) -> bool {
        use wgpu::PresentMode::*;
        match self.wgpu.surface_config.present_mode {
            AutoVsync | Fifo | FifoRelaxed => true,
            AutoNoVsync | Mailbox | Immediate => false,
        }
    }

    pub fn begin_frame(&mut self) {
        assert!(self.frame.is_none(), "called begin frame before end frame");

        let wgpu = &self.wgpu;
        let output = match wgpu.surface.get_current_texture() {
            Err(_) => {
                // Surface lost or something so reconfigure and try to reobtain
                wgpu.surface.configure(&wgpu.device, &wgpu.surface_config);

                wgpu.surface
                    .get_current_texture()
                    .expect("failed to get surface texture")
            }
            Ok(output) => output,
        };

        self.frame = Some(FrameContext {
            encoder: wgpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None }),
            output_view: output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default()),
            output,
        });
    }

    pub fn end_frame(&mut self) {
        let frame = self
            .frame
            .take()
            .expect("called end frame before begin frame");

        self.wgpu.queue.submit([frame.encoder.finish()]);
        frame.output.present();

        self.caches.uniform_buffer_allocator.free();
        self.caches.storage_buffer_allocator.free();
        self.caches.vertex_buffer_allocator.free();
        self.caches.index_buffer_allocator.free();
        self.caches.bind_group_holder.clear();
    }
}

pub struct FrameContext {
    pub encoder: wgpu::CommandEncoder,
    pub output_view: wgpu::TextureView,
    pub output: wgpu::SurfaceTexture,
}

pub struct GraphicsCaches {
    pub render_pipeline: RenderPipelineCache,
    pub bind_group_holder: Vec<wgpu::BindGroup>,
    pub uniform_buffer_allocator: DynamicBufferAllocator,
    pub storage_buffer_allocator: DynamicBufferAllocator,
    pub index_buffer_allocator: DynamicBufferAllocator,
    pub vertex_buffer_allocator: DynamicBufferAllocator,
    pub sampler: SamplerCache,
}

pub struct GraphicsData {
    pub white_texture: RefId<Texture>,
    pub square_mesh: Mesh,
    pub single_draw_shader: RefId<Shader>,
    pub instanced_shader: RefId<Shader>,
}

pub struct WGPUContext {
    pub queue: wgpu::Queue,
    pub device: wgpu::Device,
    pub surface: wgpu::Surface,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub limits: wgpu::Limits,
}

impl WGPUContext {
    async fn new(window: &Window) -> Self {
        // Init with backends from environment variables or the default
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::util::backend_bits_from_env().unwrap_or(wgpu::Backends::all()),
            ..Default::default()
        });

        // SAFETY: Surface needs to live as long as the window
        // Both window and RendererAPI exist within Engine so they should have the same lifetime
        let surface =
            unsafe { instance.create_surface(&window.win()) }.expect("Failed to create surface!");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find GPU adapter!");

        let info = adapter.get_info();
        log::info!("Using {} ({:?})", info.name, info.backend);

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .expect("Failed to request a device!");

        let surface_capabilities = surface.get_capabilities(&adapter);

        // Prefer SRGB surface formats
        let surface_format = surface_capabilities
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .unwrap_or_else(|| {
                log::warn!("SRGB surface not supported, colors will come out darker");
                &surface_capabilities.formats[0]
            });

        let size = window.size();
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            present_mode: surface_capabilities.present_modes[0],
            format: *surface_format,
            alpha_mode: surface_capabilities.alpha_modes[0],
            width: size.x,
            height: size.y,
            view_formats: Vec::default(),
        };

        surface.configure(&device, &surface_config);

        Self {
            limits: device.limits(),
            queue,
            surface_config,
            device,
            surface,
        }
    }
}
