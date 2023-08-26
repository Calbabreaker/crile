use wgpu::util::DeviceExt;

use crate::RendererAPI;

/// Buffer that uploads its data to the GPU
pub struct Buffer {
    pub(crate) buffer: wgpu::Buffer,
    pub length: usize,
    pub layout: Option<wgpu::VertexBufferLayout<'static>>,
}

impl Buffer {
    pub fn new_static<T: bytemuck::Pod>(
        api: &RendererAPI,
        usage: wgpu::BufferUsages,
        data: &[T],
    ) -> Self {
        Self {
            buffer: api
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(data),
                    usage,
                }),
            length: data.len(),
            layout: None,
        }
    }

    pub fn new_dynamic<T>(api: &RendererAPI, usage: wgpu::BufferUsages, max_length: usize) -> Self {
        Self {
            buffer: api.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: (max_length * std::mem::size_of::<T>()) as u64,
                mapped_at_creation: false,
                usage: usage | wgpu::BufferUsages::COPY_DST,
            }),
            length: max_length,
            layout: None,
        }
    }

    pub fn update<T: bytemuck::Pod>(&self, api: &RendererAPI, data: &[T]) {
        api.queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(data));
    }
}
