use crate::{window::Window, EngineError, Mesh, Texture, Vector2U};

pub struct GraphicsContext {
    pub(crate) wgpu: WGPUContext,
    pub frame: Option<FrameContext>,
    pub data: GraphicsContextData,
}

impl GraphicsContext {
    pub fn new(window: &Window) -> Self {
        let wgpu = pollster::block_on(WGPUContext::new(window));

        Self {
            data: GraphicsContextData::new(&wgpu),
            wgpu,
            frame: None,
        }
    }

    pub fn resize(&mut self, size: Vector2U) {
        let wgpu = &mut self.wgpu;
        wgpu.surface_config.width = size.x;
        wgpu.surface_config.height = size.y;
        wgpu.surface.configure(&wgpu.device, &wgpu.surface_config);
    }

    /// Tries to enable/disable vsync
    /// On some platforms disabling vsync is not possible
    pub fn set_vsync(&mut self, enable: bool) {
        let wgpu = &mut self.wgpu;
        match enable {
            true => wgpu.surface_config.present_mode = wgpu::PresentMode::AutoVsync,
            false => wgpu.surface_config.present_mode = wgpu::PresentMode::AutoNoVsync,
        }
        wgpu.surface.configure(&wgpu.device, &wgpu.surface_config);
    }

    pub fn vsync_enabled(&self) -> bool {
        use wgpu::PresentMode::*;
        match self.wgpu.surface_config.present_mode {
            AutoVsync | Fifo | FifoRelaxed => true,
            AutoNoVsync | Mailbox | Immediate => false,
        }
    }

    pub fn begin_frame(&mut self) -> Result<(), EngineError> {
        if self.frame.is_some() {
            return Err(EngineError::RenderError(
                "called begin frame before end frame".to_string(),
            ));
        }

        let wgpu = &self.wgpu;
        let output = match wgpu.surface.get_current_texture() {
            Err(_) => {
                // Surface lost or something so reconfigure and try to reobtain
                wgpu.surface.configure(&wgpu.device, &wgpu.surface_config);

                wgpu.surface.get_current_texture().map_err(|err| {
                    // If can't get again then something bad happened
                    EngineError::RenderError(format!("failed to get surface texture {err}"))
                })?
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

        Ok(())
    }

    pub fn end_frame(&mut self) -> Result<(), EngineError> {
        let frame = self.frame.take().ok_or(EngineError::RenderError(
            "called end frame before begin frame".to_string(),
        ))?;

        self.wgpu.queue.submit([frame.encoder.finish()]);
        frame.output.present();

        Ok(())
    }
}

pub struct FrameContext {
    pub encoder: wgpu::CommandEncoder,
    pub output_view: wgpu::TextureView,
    pub output: wgpu::SurfaceTexture,
}

pub struct GraphicsContextData {
    pub white_texture: Texture,
    pub square_mesh: Mesh,
    pub instance_shader: wgpu::ShaderModule,

    pub render_pipeline: crate::RenderPipeline,
    pub texture_bind_group: crate::BindGroup,
    pub uniform_bind_group: crate::BindGroup,
    pub draw_uniform_buffer: wgpu::Buffer,
}

impl GraphicsContextData {
    pub fn new(wgpu: &WGPUContext) -> Self {
        let square_mesh = Mesh::new_square(&wgpu);
        let white_texture = Texture::new(&wgpu, 1, 1, &[255, 255, 255, 255]);

        let instance_shader = wgpu
            .device
            .create_shader_module(wgpu::include_wgsl!("./instance.wgsl"));

        use crate::*;

        // Please cache me
        let texture_bind_group =
            BindGroup::new(&wgpu, &BindGroupEntry::from_texture(&white_texture));

        let draw_uniform_buffer = wgpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
            size: std::mem::size_of::<DrawUniform>() as u64,
        });

        let uniform_bind_group = BindGroup::new(
            &wgpu,
            &[BindGroupEntry::from_uniform(
                wgpu::ShaderStages::VERTEX,
                &draw_uniform_buffer,
            )],
        );

        let render_pipeline = RenderPipeline::new(
            &wgpu,
            RenderPipelineConfig {
                shader: &instance_shader,
                bind_group_layouts: &[
                    &uniform_bind_group.gpu_layout,
                    &texture_bind_group.gpu_layout,
                ],
                vertex_buffer_layouts: &[Vertex::LAYOUT],
            },
        );
        Self {
            white_texture,
            square_mesh,
            instance_shader,
            draw_uniform_buffer,
            render_pipeline,
            texture_bind_group,
            uniform_bind_group,
        }
    }
}

pub struct WGPUContext {
    pub queue: wgpu::Queue,
    pub device: wgpu::Device,
    pub surface: wgpu::Surface,
    pub surface_config: wgpu::SurfaceConfiguration,
}

impl WGPUContext {
    async fn new(window: &Window) -> Self {
        // Init with backends from environment variables or the default
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::util::backend_bits_from_env().unwrap_or(wgpu::Backends::all()),
            dx12_shader_compiler: wgpu::Dx12Compiler::default(),
        });

        // SAFETY: Surface needs to live as long as the window
        // Both window and RendererAPI exist within Engine so they should have the same lifetime
        let surface = unsafe { instance.create_surface(&window.handle()) }
            .expect("Failed to create surface!");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find GPU adapter!");

        let adapter_info = adapter.get_info();
        log::info!("Using {} ({:?})", adapter_info.name, adapter_info.backend);

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
            queue,
            surface_config,
            device,
            surface,
        }
    }
}
