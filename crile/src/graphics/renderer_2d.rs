use std::num::NonZeroU32;

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
    texture_coords: Vector2,
    color: Color,
    texture_index: u32,
}

impl Vertex {
    const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x4, 3 => Uint32],
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

// TODO: dynamically adjust max limits
const MAX_SPRITES: usize = 100000;

#[derive(Default, Clone)]
pub struct SpriteData {
    pub transform: Matrix4,
    pub color: Color,
    pub texture_index: u32,
}

#[derive(Default)]
pub struct BatchData {
    pub textures: Vec<Texture>,
    pub sprites: Vec<SpriteData>,
}

pub struct Renderer2D {
    render_pipeline: RenderPipeline,
    camera_uniform_buffer: UniformBuffer<CameraUniform>,
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u16>,
    sprite_verticies: Vec<Vertex>,
    texture_array_group_layout: wgpu::BindGroupLayout,
    camera_bind_group: BindGroup,
}

impl Renderer2D {
    pub fn new(api: &RendererAPI) -> Self {
        let vertex_buffer = VertexBuffer::new_dynamic(api, MAX_SPRITES * 4);
        let index_buffer = IndexBuffer::new_quad_index(api, MAX_SPRITES * 6);

        let camera_uniform_buffer = UniformBuffer::new_dynamic(api, 1);
        let camera_bind_group = BindGroup::new(
            api,
            &[BindGroupEntry::from_uniform(
                wgpu::ShaderStages::VERTEX,
                &camera_uniform_buffer,
            )],
        );

        let image = image::load_from_memory(include_bytes!("./test.png")).unwrap();
        let texture = Texture::from_image(api, image);

        let texture_array_group_layout =
            api.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: NonZeroU32::new(1),
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: NonZeroU32::new(1),
                        },
                    ],
                });

        let render_pipeline = RenderPipeline::new(
            api,
            RenderPipelineConfig {
                shader: wgpu::include_wgsl!("./shader.wgsl"),
                bind_group_layouts: &[&camera_bind_group.gpu_layout, &texture_array_group_layout],
                vertex_buffer_layouts: &[Vertex::LAYOUT],
            },
        );

        Self {
            render_pipeline,
            camera_uniform_buffer,
            index_buffer,
            vertex_buffer,
            sprite_verticies: vec![Vertex::default(); MAX_SPRITES * 4],
            texture_array_group_layout,
            camera_bind_group,
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

    pub fn draw_batch(
        &mut self,
        api: &RendererAPI,
        instance: &mut RenderInstance,
        batch: &BatchData,
    ) {
        let texture_array_group = api.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.texture_array_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(
                        &batch
                            .textures
                            .iter()
                            .map(|texture| &texture.gpu_view)
                            .collect::<Vec<_>>(),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::SamplerArray(
                        &batch
                            .textures
                            .iter()
                            .map(|texture| &texture.gpu_sampler)
                            .collect::<Vec<_>>(),
                    ),
                },
            ],
        });

        for (sprite_number, sprite) in batch.sprites.iter().enumerate() {
            for i in 0..4 {
                let vertex = &mut self.sprite_verticies[sprite_number * 4 + i];
                let position = sprite.transform.mul_vec4(VERTEX_POSITIONS[i]);
                vertex.position = Vector2::new(position.x, position.y);
                vertex.texture_coords = VERTEX_UVS[i];
                vertex.color = sprite.color;
                vertex.texture_index = sprite.texture_index;
            }
        }

        let mut render_pass = instance.begin_render_pass(Some(Color::from_rgb(0, 0, 0)));
        let vertices_slice = &self.sprite_verticies[0..batch.sprites.len() * 4];
        self.vertex_buffer.update(api, vertices_slice);

        self.render_pipeline.bind(&mut render_pass);
        self.index_buffer.bind(&mut render_pass);
        self.vertex_buffer.bind(&mut render_pass, 0);
        render_pass.set_bind_group(0, &self.camera_bind_group.gpu_group, &[]);
        render_pass.set_bind_group(1, &texture_array_group, &[]);
        render_pass.draw_indexed(0..(batch.sprites.len() * 6) as u32, 0, 0..1);
    }
}
