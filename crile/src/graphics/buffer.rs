use wgpu::util::DeviceExt;

use crate::RendererAPI;

pub enum BufferKind {
    Vertex,
    Index,
    Uniform,
}

impl BufferKind {
    fn get_usage(&self) -> wgpu::BufferUsages {
        match self {
            BufferKind::Vertex => wgpu::BufferUsages::VERTEX,
            BufferKind::Index => wgpu::BufferUsages::INDEX,
            BufferKind::Uniform => wgpu::BufferUsages::UNIFORM,
        }
    }
}

/// Buffer that uploads its data to the GPU
pub struct Buffer<T> {
    pub(crate) gpu_buffer: wgpu::Buffer,
    pub length: usize,
    kind: BufferKind,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: bytemuck::Pod> Buffer<T> {
    pub fn new_static(api: &RendererAPI, kind: BufferKind, data: &[T]) -> Self {
        Self {
            gpu_buffer: api
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(data),
                    usage: kind.get_usage(),
                }),
            length: data.len(),
            kind,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn new_dynamic(api: &RendererAPI, kind: BufferKind, max_length: usize) -> Self {
        Self {
            gpu_buffer: api.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: (max_length * std::mem::size_of::<T>()) as u64,
                mapped_at_creation: false,
                usage: kind.get_usage() | wgpu::BufferUsages::COPY_DST,
            }),
            length: max_length,
            kind,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn update(&self, api: &RendererAPI, data: &[T]) {
        api.queue
            .write_buffer(&self.gpu_buffer, 0, bytemuck::cast_slice(data));
    }

    pub fn bind<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, slot: u32) {
        match self.kind {
            BufferKind::Vertex => render_pass.set_vertex_buffer(slot, self.gpu_buffer.slice(..)),
            BufferKind::Index => {
                render_pass.set_index_buffer(self.gpu_buffer.slice(..), self.get_index_format())
            }
            BufferKind::Uniform => panic!("uniform buffer can not be bound to render pass"),
        }
    }

    fn get_index_format(&self) -> wgpu::IndexFormat {
        use std::any::TypeId;
        if TypeId::of::<T>() == TypeId::of::<u32>() {
            wgpu::IndexFormat::Uint32
        } else if TypeId::of::<T>() == TypeId::of::<u16>() {
            wgpu::IndexFormat::Uint16
        } else {
            panic!("only u16 or u32 expected");
        }
    }
}
