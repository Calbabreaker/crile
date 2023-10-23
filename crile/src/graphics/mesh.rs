use wgpu::util::DeviceExt;

use super::WGPUContext;

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshVertex {
    pub position: [f32; 2],
    pub texture_coords: [f32; 2],
    pub color: [f32; 4],
}

impl MeshVertex {
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<MeshVertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x4],
    };
}

#[derive(Debug)]
pub struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

impl Mesh {
    pub fn new(wgpu: &WGPUContext, vertices: &[MeshVertex], indicies: &[u32]) -> Self {
        Self {
            vertex_buffer: Self::create_buffer(wgpu, wgpu::BufferUsages::VERTEX, vertices),
            index_buffer: Self::create_buffer(wgpu, wgpu::BufferUsages::INDEX, indicies),
            index_count: indicies.len() as u32,
        }
    }

    pub fn new_square(wgpu: &WGPUContext) -> Self {
        Self::new(
            wgpu,
            &[
                MeshVertex {
                    position: [0., 0.],
                    // Flip the texture coords in the y direction to flip the texture
                    texture_coords: [0., 1.],
                    color: [1.; 4],
                },
                MeshVertex {
                    position: [1., 0.],
                    texture_coords: [1., 1.],
                    color: [1.; 4],
                },
                MeshVertex {
                    position: [1., 1.],
                    texture_coords: [1., 0.],
                    color: [1.; 4],
                },
                MeshVertex {
                    position: [0.0, 1.0],
                    texture_coords: [0., 0.],
                    color: [1.; 4],
                },
            ],
            &[0, 1, 2, 2, 3, 0],
        )
    }

    fn create_buffer<T: bytemuck::Pod>(
        wgpu: &WGPUContext,
        usage: wgpu::BufferUsages,
        data: &[T],
    ) -> wgpu::Buffer {
        wgpu.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                usage,
                contents: bytemuck::cast_slice(data),
            })
    }
}
