use std::sync::Arc;

use crate::{
    DynamicBufferAllocator, Mesh, RefId, RenderPipelineCache, SamplerCache, Shader, ShaderKind,
    Texture, Window, WindowId,
};

pub struct GraphicsContext {
    pub wgpu: WgpuContext,
    pub frame: Option<FrameContext>,
    pub data: GraphicsData,
    pub caches: GraphicsCaches,
}

impl GraphicsContext {
    pub(crate) fn new(winit: &Arc<winit::window::Window>) -> Self {
        let wgpu = pollster::block_on(WgpuContext::new(winit));

        Self {
            data: GraphicsData::new(&wgpu),
            caches: GraphicsCaches::new(&wgpu),
            wgpu,
            frame: None,
        }
    }

    pub fn begin_frame(&mut self, window: &Window) {
        assert!(self.frame.is_none(), "called begin frame before end frame");

        let wgpu = &self.wgpu;
        let output = wgpu.get_surface_texture(window.id());

        self.frame = Some(FrameContext {
            window_id: window.id(),
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

    pub fn target_window_id(&self) -> Option<WindowId> {
        self.frame.as_ref().map(|frame| frame.window_id)
    }
}

pub struct FrameContext {
    pub encoder: wgpu::CommandEncoder,
    pub output_view: wgpu::TextureView,
    pub output: wgpu::SurfaceTexture,
    pub window_id: WindowId,
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

impl GraphicsCaches {
    pub(crate) fn new(wgpu: &WgpuContext) -> Self {
        Self {
            bind_group_holder: Vec::new(),
            render_pipeline: RenderPipelineCache::default(),
            sampler: SamplerCache::default(),
            uniform_buffer_allocator: DynamicBufferAllocator::new(
                wgpu,
                wgpu::BufferUsages::UNIFORM,
            ),
            storage_buffer_allocator: DynamicBufferAllocator::new(
                wgpu,
                wgpu::BufferUsages::STORAGE,
            ),
            vertex_buffer_allocator: DynamicBufferAllocator::new(wgpu, wgpu::BufferUsages::VERTEX),
            index_buffer_allocator: DynamicBufferAllocator::new(wgpu, wgpu::BufferUsages::INDEX),
        }
    }
}

pub struct GraphicsData {
    pub white_texture: RefId<Texture>,
    pub square_mesh: Mesh,
    pub single_draw_shader: RefId<Shader>,
    pub instanced_shader: RefId<Shader>,
}

impl GraphicsData {
    pub(crate) fn new(wgpu: &WgpuContext) -> Self {
        Self {
            square_mesh: Mesh::new_square(wgpu),
            white_texture: Texture::from_pixels(wgpu, glam::UVec2::ONE, &[255, 255, 255, 255])
                .into(),
            instanced_shader: RefId::new(Shader::new(
                wgpu,
                wgpu::include_wgsl!("./shaders/instanced.wgsl"),
                ShaderKind::Instanced,
            )),
            single_draw_shader: RefId::new(Shader::new(
                wgpu,
                wgpu::include_wgsl!("./shaders/single_draw.wgsl"),
                ShaderKind::DrawSingle,
            )),
        }
    }
}

pub struct WindowViewport {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
}

pub struct WgpuContext {
    pub queue: wgpu::Queue,
    pub device: wgpu::Device,
    pub adapter: wgpu::Adapter,
    pub instance: wgpu::Instance,
    pub limits: wgpu::Limits,
    pub viewport_map: hashbrown::HashMap<WindowId, WindowViewport>,
}

impl WgpuContext {
    pub(crate) async fn new(winit: &Arc<winit::window::Window>) -> Self {
        // Init with backends from environment variables or the default
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::util::backend_bits_from_env().unwrap_or(wgpu::Backends::all()),
            ..Default::default()
        });

        let surface = instance
            .create_surface(winit.clone())
            .expect("Failed to create surface!");

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
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .expect("Failed to request a device!");

        let mut wgpu = Self {
            viewport_map: hashbrown::HashMap::default(),
            limits: device.limits(),
            queue,
            device,
            instance,
            adapter,
        };
        wgpu.add_surface(winit, surface);
        wgpu
    }

    fn add_surface(&mut self, winit: &winit::window::Window, surface: wgpu::Surface<'static>) {
        let size = winit.inner_size();
        let config = surface
            .get_default_config(&self.adapter, size.width, size.height)
            .unwrap();
        surface.configure(&self.device, &config);

        self.viewport_map
            .insert(winit.id(), WindowViewport { surface, config });
    }

    pub(crate) fn new_viewport(&mut self, window: &Window) {
        self.add_surface(
            &window.winit,
            self.instance
                .create_surface(window.winit.clone())
                .expect("Failed to create surface!"),
        );
    }

    pub(crate) fn delete_viewport(&mut self, window_id: WindowId) {
        self.viewport_map.remove(&window_id);
    }

    pub(crate) fn resize_viewport(&mut self, size: glam::UVec2, window_id: WindowId) {
        let viewport = self.viewport_map.get_mut(&window_id).unwrap();
        viewport.config.width = size.x;
        viewport.config.height = size.y;
        viewport.surface.configure(&self.device, &viewport.config);
    }

    fn get_surface_texture(&self, window_id: WindowId) -> wgpu::SurfaceTexture {
        let viewport = self.viewport_map.get(&window_id).unwrap();
        match viewport.surface.get_current_texture() {
            Err(_) => {
                // Surface lost or something so reconfigure and try to reobtain
                viewport.surface.configure(&self.device, &viewport.config);

                viewport
                    .surface
                    .get_current_texture()
                    .expect("failed to get surface texture")
            }
            Ok(output) => output,
        }
    }
}
