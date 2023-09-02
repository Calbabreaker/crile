use wgpu::util::DeviceExt;

use crate::RendererAPI;

macro_rules! create_buffer_type {
    ($name: ident, $usage: ident $(, $restraint:path )?) => {
        pub struct $name<T> {
            pub gpu_buffer: wgpu::Buffer,
            _phantom: std::marker::PhantomData<T>,
        }

        impl<T> $name<T>
        where
            T: bytemuck::Pod + $($restraint)*,
        {
            pub fn new_static(api: &RendererAPI, data: &[T]) -> Self {
                Self {
                    gpu_buffer: api
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: None,
                            contents: bytemuck::cast_slice(data),
                            usage: wgpu::BufferUsages::$usage,
                        }),
                    _phantom: std::marker::PhantomData,
                }
            }

            pub fn new_dynamic(api: &RendererAPI, max_length: usize) -> Self {
                Self {
                    gpu_buffer: api.device.create_buffer(&wgpu::BufferDescriptor {
                        label: None,
                        size: (max_length * std::mem::size_of::<T>()) as u64,
                        mapped_at_creation: false,
                        usage: wgpu::BufferUsages::$usage | wgpu::BufferUsages::COPY_DST,
                    }),
                    _phantom: std::marker::PhantomData,
                }
            }

            pub fn update(&self, api: &RendererAPI, data: &[T]) {
                api.queue
                    .write_buffer(&self.gpu_buffer, 0, bytemuck::cast_slice(data));
            }

        }
    };
}

create_buffer_type!(VertexBuffer, VERTEX);
create_buffer_type!(UniformBuffer, UNIFORM);
create_buffer_type!(IndexBuffer, INDEX, Indexable);

impl<T: bytemuck::Pod> VertexBuffer<T> {
    pub fn bind<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, slot: u32) {
        render_pass.set_vertex_buffer(slot, self.gpu_buffer.slice(..));
    }
}

pub trait Indexable {
    fn get_index_format() -> wgpu::IndexFormat;
}

impl Indexable for u16 {
    fn get_index_format() -> wgpu::IndexFormat {
        wgpu::IndexFormat::Uint16
    }
}

impl Indexable for u32 {
    fn get_index_format() -> wgpu::IndexFormat {
        wgpu::IndexFormat::Uint32
    }
}

impl<T: Indexable + bytemuck::Pod> IndexBuffer<T> {
    pub fn bind<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_index_buffer(self.gpu_buffer.slice(..), T::get_index_format())
    }
}
