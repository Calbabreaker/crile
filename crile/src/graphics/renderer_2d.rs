use crate::{
    BindGroup, BindGroupEntry, Camera, Color, IndexBuffer, Matrix4, RenderInstance, RenderPipeline,
    RenderPipelineConfig, RendererAPI, Texture, UniformBuffer, Vector2, Vector4, VertexBuffer,
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
    color: Color,
}

impl Vertex {
    const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x4],
    };
}

const VERTEX_POSITIONS: &[Vector4] = &[
    Vector4::new(-0.5, -0.5, 0.0, 1.0),
    Vector4::new(0.5, -0.5, 0.0, 1.0),
    Vector4::new(0.5, 0.5, 0.0, 1.0),
    Vector4::new(-0.5, 0.5, 0.0, 1.0),
];

const VERTEX_UVS: &[Vector2] = &[
    Vector2::new(0.0, 1.0),
    Vector2::new(1.0, 1.0),
    Vector2::new(1.0, 0.0),
    Vector2::new(0.0, 0.0),
];

const MAX_QUADS: usize = 100;

pub struct Renderer2D {
    render_pipeline: RenderPipeline,
    camera_uniform_buffer: UniformBuffer<CameraUniform>,
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u16>,
    quad_verticies: Vec<Vertex>,
    quads_in_batch: usize,
}

impl Renderer2D {
    pub fn new(api: &RendererAPI) -> Self {
        let vertex_buffer = VertexBuffer::new_dynamic(api, MAX_QUADS * 4);
        let index_buffer = IndexBuffer::new_quad_index(api, MAX_QUADS * 6);

        let camera_uniform_buffer = UniformBuffer::new_dynamic(api, 1);
        let camera_bind_group = BindGroup::new(
            api,
            &[BindGroupEntry::from_uniform(
                wgpu::ShaderStages::VERTEX,
                &camera_uniform_buffer,
            )],
        );

        let image = image::load_from_memory(include_bytes!("./test.png")).unwrap();
        let texture = Texture::new(api, image);
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
            quad_verticies: vec![Vertex::default(); MAX_QUADS * 4],
            quads_in_batch: 0,
        }
    }

    pub fn begin(&mut self, api: &RendererAPI, camera: &Camera) {
        self.camera_uniform_buffer.update(
            api,
            &[CameraUniform {
                view_projection: camera.get_projection(),
            }],
        );
    }

    /// Adds a quad to the batch the be rendered
    /// If the batch is full, it will render immediatly to free the batch
    pub fn draw_quad(&mut self, transform: &Matrix4, color: &Color) {
        for i in 0..4 {
            let vertex = &mut self.quad_verticies[self.quads_in_batch * 4 + i];
            let position = transform.mul_vec4(VERTEX_POSITIONS[i]);
            vertex.position = Vector2::new(position.x, position.y);
            vertex.uvs = VERTEX_UVS[i];
            vertex.color = *color;
        }

        self.quads_in_batch += 1;
    }

    pub fn flush(&mut self, api: &RendererAPI, instance: &mut RenderInstance) {
        let mut render_pass = instance.begin_render_pass(Some(Color::from_rgb(0, 0, 0)));
        let vertices_slice = &self.quad_verticies[0..self.quads_in_batch * 4];
        self.vertex_buffer.update(api, vertices_slice);

        self.render_pipeline.bind(&mut render_pass);
        self.index_buffer.bind(&mut render_pass);
        self.vertex_buffer.bind(&mut render_pass, 0);
        render_pass.draw_indexed(0..(self.quads_in_batch * 6) as u32, 0, 0..1);

        self.quads_in_batch = 0;
    }
}
