use wgpu::util::DeviceExt;

use super::WgpuContext;

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
    pub fn new(wgpu: &WgpuContext, vertices: &[MeshVertex], indicies: &[u32]) -> Self {
        Self {
            vertex_buffer: Self::create_buffer(wgpu, wgpu::BufferUsages::VERTEX, vertices),
            index_buffer: Self::create_buffer(wgpu, wgpu::BufferUsages::INDEX, indicies),
            index_count: indicies.len() as u32,
        }
    }

    pub fn new_square(wgpu: &WgpuContext) -> Self {
        Self::new(
            wgpu,
            &[
                MeshVertex {
                    position: [-0.5, -0.5],
                    texture_coords: [0., 0.],
                    color: [1.; 4],
                },
                MeshVertex {
                    position: [0.5, -0.5],
                    texture_coords: [1., 0.],
                    color: [1.; 4],
                },
                MeshVertex {
                    position: [0.5, 0.5],
                    texture_coords: [1., 1.],
                    color: [1.; 4],
                },
                MeshVertex {
                    position: [-0.5, 0.5],
                    texture_coords: [0., 1.],
                    color: [1.; 4],
                },
            ],
            &[0, 1, 2, 2, 3, 0],
        )
    }

    fn create_buffer<T: bytemuck::Pod>(
        wgpu: &WgpuContext,
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

    pub fn view(&self) -> MeshView {
        MeshView::new(
            self.vertex_buffer.slice(..),
            self.index_buffer.slice(..),
            self.index_count,
        )
    }
}

pub struct MeshView<'a> {
    pub vertex_buffer: wgpu::BufferSlice<'a>,
    pub index_buffer: wgpu::BufferSlice<'a>,
    pub index_count: u32,
}

impl<'a> MeshView<'a> {
    pub fn new(
        vertex_buffer: wgpu::BufferSlice<'a>,
        index_buffer: wgpu::BufferSlice<'a>,
        index_count: u32,
    ) -> Self {
        Self {
            vertex_buffer,
            index_buffer,
            index_count,
        }
    }
}
