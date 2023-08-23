use crate::{window::Window, Vector2U};

pub struct RenderInstance {
    encoder: wgpu::CommandEncoder,
    view: wgpu::TextureView,
    output: wgpu::SurfaceTexture,
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
        let backends = wgpu::util::backend_bits_from_env().unwrap_or(wgpu::Backends::all());
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
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

        let size = window.size();

        let surface_capabilities = surface.get_capabilities(&adapter);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            present_mode: surface_capabilities.present_modes[0],
            format: surface_capabilities.formats[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            width: size.x as u32,
            height: size.y as u32,
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
        self.config.width = size.x as u32;
        self.config.height = size.y as u32;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn begin_frame(&self) -> Option<RenderInstance> {
        match self.surface.get_current_texture() {
            Err(wgpu::SurfaceError::OutOfMemory) => panic!("GPU out of memory"),
            Err(wgpu::SurfaceError::Lost) => {
                self.surface.configure(&self.device, &self.config);
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
        self.queue
            .submit(std::iter::once(render_instance.encoder.finish()));
        render_instance.output.present();
    }

    pub fn begin_render_pass<'a>(
        &'a self,
        render_instance: &'a mut RenderInstance,
    ) -> wgpu::RenderPass {
        render_instance
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &render_instance.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            })
    }
}
