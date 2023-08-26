use crate::{
    BindGroup, BindGroupEntry, Buffer, Camera, Matrix4, RenderPipeline, RenderPipelineConfig,
    Texture, Vector2,
};

use super::renderer_api::{RenderInstance, RendererAPI};

#[repr(C)]
#[derive(Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_projection: Matrix4,
}

#[repr(C)]
#[derive(Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: Vector2,
    uvs: Vector2,
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
        uvs: Vector2::new(0.0, 1.0),
    },
    Vertex {
        position: Vector2::new(0.5, -0.5),
        uvs: Vector2::new(1.0, 1.0),
    },
    Vertex {
        position: Vector2::new(0.5, 0.5),
        uvs: Vector2::new(1.0, 0.0),
    },
    Vertex {
        position: Vector2::new(-0.5, 0.5),
        uvs: Vector2::new(0.0, 0.0),
    },
];

const INDICIES: &[u16] = &[0, 1, 2, 2, 3, 0];

pub struct Renderer2D {
    render_pipeline: RenderPipeline,
    camera_uniform_buffer: Buffer,
}

impl Renderer2D {
    pub fn new(api: &RendererAPI) -> Self {
        let mut vertex_buffer = Buffer::new_static(api, wgpu::BufferUsages::VERTEX, VERTICES);
        vertex_buffer.layout = Some(Vertex::LAYOUT);
        let index_buffer = Buffer::new_static(api, wgpu::BufferUsages::INDEX, INDICIES);

        let camera_uniform_buffer =
            Buffer::new_dynamic::<CameraUniform>(api, wgpu::BufferUsages::UNIFORM, 1);
        let camera_bind_group = BindGroup::new(
            api,
            &[BindGroupEntry::from_buffer(
                wgpu::ShaderStages::VERTEX,
                &camera_uniform_buffer,
            )],
        );

        let image = image::load_from_memory(include_bytes!("./test.png")).unwrap();
        let texture = Texture::new(&api, image);
        let texture_bind_group = BindGroup::new(api, &BindGroupEntry::from_texture(&texture));

        let render_pipeline = RenderPipeline::new(
            api,
            RenderPipelineConfig {
                shader: wgpu::include_wgsl!("./shader.wgsl"),
                bind_groups: vec![camera_bind_group, texture_bind_group],
                index_buffer,
                vertex_buffers: vec![vertex_buffer],
            },
        );

        Self {
            render_pipeline,
            camera_uniform_buffer,
        }
    }

    pub fn begin(&self, api: &RendererAPI, camera: &Camera) {
        self.camera_uniform_buffer.update(
            api,
            &[CameraUniform {
                view_projection: camera.get_projection(),
            }],
        );
    }

    pub fn render(&self, instance: &mut RenderInstance) {
        self.render_pipeline.draw_indexed(instance, &INDICIES);
    }
}
