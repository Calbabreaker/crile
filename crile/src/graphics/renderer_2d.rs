use crate::{
    BindGroup, BindGroupEntry, Camera, Color, GraphicsContext, Matrix4, Mesh, RenderInstance,
    RenderPipeline, RenderPipelineConfig, Texture, UniformBuffer, Vertex,
};

#[repr(C)]
#[derive(Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_projection: Matrix4,
}

#[derive(Default, Clone)]
pub struct SpriteData {
    pub transform: Matrix4,
    pub color: Color,
    pub texture_index: u32,
}

// #[derive(Default)]
// pub struct BatchData {
//     pub sprites: Vec<SpriteData>,
//     bind_group: BindGroup,
// }

// impl BatchData {
//     pub fn update_textures(&mut self, textures: &Vec<&Texture>) {}
// }

pub struct Renderer2D {
    render_pipeline: RenderPipeline,
    camera_uniform_buffer: UniformBuffer<CameraUniform>,
    square_mesh: Mesh,
    camera_bind_group: BindGroup,
    texture_bind_group: BindGroup,
}

impl Renderer2D {
    pub fn new(gfx: &GraphicsContext) -> Self {
        let square_mesh = Mesh::new_square(gfx);

        let camera_uniform_buffer = UniformBuffer::new_dynamic(gfx, 1);
        let camera_bind_group = BindGroup::new(
            gfx,
            &[BindGroupEntry::from_uniform(
                wgpu::ShaderStages::VERTEX,
                &camera_uniform_buffer,
            )],
        );

        // let image = image::open("assets/test.png").unwrap();
        // let texture = Texture::from_image(gfx, image);
        let texture = Texture::new(gfx, 1, 1, &[255, 255, 255, 255]);
        let texture_bind_group = BindGroup::new(gfx, &BindGroupEntry::from_texture(&texture));

        let render_pipeline = RenderPipeline::new(
            gfx,
            RenderPipelineConfig {
                shader: wgpu::include_wgsl!("./instance.wgsl"),
                bind_group_layouts: &[
                    &camera_bind_group.gpu_layout,
                    &texture_bind_group.gpu_layout,
                ],
                vertex_buffer_layouts: &[Vertex::LAYOUT],
            },
        );

        Self {
            render_pipeline,
            camera_uniform_buffer,
            square_mesh,
            camera_bind_group,
            texture_bind_group,
        }
    }

    pub fn begin(&mut self, gfx: &GraphicsContext, camera: &Camera) {
        self.camera_uniform_buffer.update(
            gfx,
            &[CameraUniform {
                view_projection: camera.get_projection(),
            }],
        );
    }

    pub fn draw_instances(&mut self, gfx: &GraphicsContext, instance: &mut RenderInstance) {
        let mut render_pass = instance.begin_render_pass(Some(Color::from_rgb(0, 0, 0)));

        render_pass.set_pipeline(&self.render_pipeline.gpu_pipeline);
        render_pass.set_index_buffer(
            self.square_mesh.index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );
        render_pass.set_vertex_buffer(0, self.square_mesh.vertex_buffer.slice(..));
        render_pass.set_bind_group(0, &self.camera_bind_group.gpu_group, &[]);
        render_pass.set_bind_group(1, &self.texture_bind_group.gpu_group, &[]);
        render_pass.draw_indexed(0..self.square_mesh.index_count, 0, 0..10);
    }
}
