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
    pub fn new() -> Self {
        let wgpu = pollster::block_on(WGPUContext::new());

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

        let white_texture = Texture::from_pixels(&wgpu, glam::UVec2::ONE, &[255, 255, 255, 255]);
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

    pub fn begin_frame(&mut self, window: &Window) {
        assert!(self.frame.is_none(), "called begin frame before end frame");

        let wgpu = &self.wgpu;
        let output = window.viewport.get_texture(wgpu);

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

impl Default for GraphicsContext {
    fn default() -> Self {
        Self::new()
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
    pub adapter: wgpu::Adapter,
    pub instance: wgpu::Instance,
    pub limits: wgpu::Limits,
}

impl WGPUContext {
    async fn new() -> Self {
        // Init with backends from environment variables or the default
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::util::backend_bits_from_env().unwrap_or(wgpu::Backends::all()),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                // compatible_surface: Some(&surface),
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find GPU adapter!");

        let info = adapter.get_info();
        log::info!("Using {} ({:?})", info.name, info.backend);

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .expect("Failed to request a device!");

        Self {
            limits: device.limits(),
            queue,
            device,
            instance,
            adapter,
        }
    }
}
