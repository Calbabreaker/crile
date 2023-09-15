use wgpu::util::DeviceExt;

use crate::GraphicsContext;

#[repr(C)]
#[derive(Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 2],
    texture_coords: [f32; 2],
}

impl Vertex {
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2],
    };
}

// Vertices and indicies stored on the gpu
pub struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

impl Mesh {
    pub fn new(gfx: &GraphicsContext, vertices: &[Vertex], indicies: &[u16]) -> Self {
        Self {
            vertex_buffer: Self::create_buffer(gfx, wgpu::BufferUsages::VERTEX, &vertices),
            index_buffer: Self::create_buffer(gfx, wgpu::BufferUsages::INDEX, &indicies),
            index_count: indicies.len() as u32,
        }
    }

    pub fn new_square(gfx: &GraphicsContext) -> Self {
        // Vector4::new(-0.5, -0.5, 0.0, 1.0),
        // Vector4::new(0.5, -0.5, 0.0, 1.0),
        // Vector4::new(0.5, 0.5, 0.0, 1.0),
        // Vector4::new(-0.5, 0.5, 0.0, 1.0),
        // Vector2::new(0.0, 1.0),
        // Vector2::new(1.0, 1.0),
        // Vector2::new(1.0, 0.0),
        // Vector2::new(0.0, 0.0),
        Self::new(
            gfx,
            &[
                Vertex {
                    position: [0.0, 0.0],
                    texture_coords: [0.0, 1.0],
                },
                Vertex {
                    position: [1.0, 0.0],
                    texture_coords: [1.0, 1.0],
                },
                Vertex {
                    position: [1.0, 1.0],
                    texture_coords: [1.0, 0.0],
                },
                Vertex {
                    position: [0.0, 1.0],
                    texture_coords: [0.0, 0.0],
                },
            ],
            &[0, 1, 2, 2, 3, 0],
        )
    }

    fn create_buffer<T: bytemuck::Pod>(
        gfx: &GraphicsContext,
        usage: wgpu::BufferUsages,
        data: &[T],
    ) -> wgpu::Buffer {
        gfx.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                usage,
                contents: bytemuck::cast_slice(data),
            })
    }
}
