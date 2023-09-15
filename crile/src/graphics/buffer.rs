use wgpu::util::DeviceExt;

use crate::GraphicsContext;

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
            pub fn new_static(gfx: &GraphicsContext, data: &[T]) -> Self {
                Self {
                    gpu_buffer: gfx
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: None,
                            contents: bytemuck::cast_slice(data),
                            usage: wgpu::BufferUsages::$usage,
                        }),
                    _phantom: std::marker::PhantomData,
                }
            }

            pub fn new_dynamic(gfx: &GraphicsContext, max_length: usize) -> Self {
                Self {
                    gpu_buffer: gfx.device.create_buffer(&wgpu::BufferDescriptor {
                        label: None,
                        size: (max_length * std::mem::size_of::<T>()) as u64,
                        mapped_at_creation: false,
                        usage: wgpu::BufferUsages::$usage | wgpu::BufferUsages::COPY_DST,
                    }),
                    _phantom: std::marker::PhantomData,
                }
            }

            pub fn update(&self, gfx: &GraphicsContext, data: &[T]) {
                gfx.queue
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
    fn from_usize(x: usize) -> Self;
}

macro_rules! impl_indexable {
    ($bit:ident, $format:ident) => {
        impl Indexable for $bit {
            fn get_index_format() -> wgpu::IndexFormat {
                wgpu::IndexFormat::$format
            }

            fn from_usize(x: usize) -> Self {
                x as $bit
            }
        }
    };
}

impl_indexable!(u32, Uint32);
impl_indexable!(u16, Uint16);

impl<T: Indexable + bytemuck::Pod> IndexBuffer<T> {
    pub fn bind<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_index_buffer(self.gpu_buffer.slice(..), T::get_index_format())
    }
}
