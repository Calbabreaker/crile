use wgpu::util::DeviceExt;

use crate::Vector2;

use super::renderer_api::{RenderInstance, RendererAPI};

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
