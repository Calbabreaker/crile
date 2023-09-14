use crate::{window::Window, Color, Vector2U};

pub struct RenderInstance {
    encoder: wgpu::CommandEncoder,
    view: wgpu::TextureView,
    output: wgpu::SurfaceTexture,
}

impl RenderInstance {
    pub fn begin_render_pass(&mut self, color: Option<Color>) -> wgpu::RenderPass {
        self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: color.map_or(wgpu::LoadOp::Load, |c| wgpu::LoadOp::Clear(c.into())),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        })
    }
}

pub struct RendererAPI {
    pub(crate) queue: wgpu::Queue,
    pub(crate) config: wgpu::SurfaceConfiguration,
    pub(crate) device: wgpu::Device,
    surface: wgpu::Surface,
}

impl RendererAPI {
    pub async fn new(window: &Window) -> Self {
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
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            present_mode: surface_capabilities.present_modes[0],
            format: *surface_format,
            alpha_mode: surface_capabilities.alpha_modes[0],
            width: size.x,
            height: size.y,
            view_formats: Vec::default(),
        };

        surface.configure(&device, &config);

        Self {
            queue,
            config,
            device,
            surface,
        }
    }

    pub fn resize(&mut self, size: Vector2U) {
        self.config.width = size.x;
        self.config.height = size.y;
        self.reconfigure();
    }

    /// Tries to enable/disable vsync
    /// On some platforms disabling vsync is not possible
    pub fn set_vsync(&mut self, enable: bool) {
        match enable {
            true => self.config.present_mode = wgpu::PresentMode::AutoVsync,
            false => self.config.present_mode = wgpu::PresentMode::AutoNoVsync,
        }
        self.reconfigure();
    }

    pub fn vsync_enabled(&self) -> bool {
        use wgpu::PresentMode::*;
        match self.config.present_mode {
            AutoVsync | Fifo | FifoRelaxed => true,
            AutoNoVsync | Mailbox | Immediate => false,
        }
    }

    pub fn begin_frame(&self) -> Option<RenderInstance> {
        match self.surface.get_current_texture() {
            Err(wgpu::SurfaceError::OutOfMemory) => panic!("GPU out of memory"),
            Err(wgpu::SurfaceError::Lost) => {
                self.reconfigure();
                None
            }
            Err(_) => None,
            Ok(output) => Some(RenderInstance {
                encoder: self
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None }),
                view: output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default()),
                output,
            }),
        }
    }

    pub fn present_frame(&self, render_instance: RenderInstance) {
        self.queue.submit([render_instance.encoder.finish()]);
        render_instance.output.present();
    }

    fn reconfigure(&self) {
        self.surface.configure(&self.device, &self.config);
    }
}
