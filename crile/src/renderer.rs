use wgpu::{util::DeviceExt, VERTEX_STRIDE_ALIGNMENT};

use crate::{window::Window, Vector2};

pub struct RenderInstance {
    encoder: wgpu::CommandEncoder,
    view: wgpu::TextureView,
    output: wgpu::SurfaceTexture,
}

pub struct RendererAPI {
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
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

        // SAFETY
        // Surface needs to live as long as the window
        // Both window and MainRenderer exist within Engine so they should have the same lifetime
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
                    // WebGL doesn't support all of wgpu's features
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

    pub fn resize(&mut self, size: Vector2) {
        self.config.width = size.x as u32;
        self.config.height = size.y as u32;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn begin_frame(&self) -> Result<RenderInstance, wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        Ok(RenderInstance {
            encoder: self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None }),
            view: output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default()),
            output,
        })
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

#[repr(C)]
#[derive(Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: crate::Vector2,
    uvs: crate::Vector2,
}

impl Vertex {
    const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2],
    };
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: Vector2::new(-0.5, -0.5),
        uvs: Vector2::new(0.0, 0.0),
    },
    Vertex {
        position: Vector2::new(0.5, -0.5),
        uvs: Vector2::new(1.0, 0.0),
    },
    Vertex {
        position: Vector2::new(0.5, 0.5),
        uvs: Vector2::new(1.0, 1.0),
    },
    Vertex {
        position: Vector2::new(-0.5, 0.5),
        uvs: Vector2::new(0.0, 1.0),
    },
];

const INDICIES: &[u32] = &[0, 1, 2, 2, 3, 0];

pub struct Renderer {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl Renderer {
    pub fn new(api: &RendererAPI) -> Self {
        let vertex_buffer = api
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = api
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(INDICIES),
                usage: wgpu::BufferUsages::INDEX,
            });

        let shader = api
            .device
            .create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let render_pipeline_layout =
            api.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                });

        let render_pipeline = api
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::LAYOUT],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: api.config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                depth_stencil: None,
                multiview: None,
            });

        Self {
            vertex_buffer,
            render_pipeline,
            index_buffer,
        }
    }

    pub fn render(&self, instance: &mut RenderInstance, renderer_api: &RendererAPI) {
        let mut render_pass = renderer_api.begin_render_pass(instance);
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..INDICIES.len() as u32, 0, 0..1);
    }
}
