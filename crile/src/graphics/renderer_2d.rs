use crate::{
    BindGroup, BindGroupEntry, Buffer, BufferKind, Camera, Matrix4, RenderInstance, RenderPipeline,
    RenderPipelineConfig, RendererAPI, Texture, Vector2,
};

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
    camera_uniform_buffer: Buffer<CameraUniform>,
    vertex_buffer: Buffer<Vertex>,
    index_buffer: Buffer<u16>,
}

impl Renderer2D {
    pub fn new(api: &RendererAPI) -> Self {
        let vertex_buffer = Buffer::new_static(api, BufferKind::Vertex, VERTICES);
        let index_buffer = Buffer::new_static(api, BufferKind::Index, INDICIES);

        let camera_uniform_buffer = Buffer::new_dynamic(api, BufferKind::Uniform, 1);
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
                vertex_buffer_layouts: &[Vertex::LAYOUT],
            },
        );

        Self {
            render_pipeline,
            camera_uniform_buffer,
            index_buffer,
            vertex_buffer,
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
        let mut render_pass = instance.begin_render_pass();
        self.render_pipeline.bind(&mut render_pass);
        self.index_buffer.bind(&mut render_pass, 0);
        self.vertex_buffer.bind(&mut render_pass, 0);
        render_pass.draw_indexed(0..INDICIES.len() as u32, 0, 0..1)
    }
}
