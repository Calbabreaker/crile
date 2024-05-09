use super::WgpuContext;
use crate::RefId;

#[derive(Debug)]
struct DynamicBufferSpace {
    buffer: RefId<wgpu::Buffer>,
    cursor: u64,
}

impl DynamicBufferSpace {
    fn new(wgpu: &WgpuContext, descriptor: &wgpu::BufferDescriptor) -> Self {
        Self {
            buffer: wgpu.device.create_buffer(descriptor).into(),
            cursor: 0,
        }
    }
}

#[derive(Debug)]
pub struct BufferAllocation {
    pub offset: u64,
    pub size: u64,
    pub buffer: RefId<wgpu::Buffer>,
}

impl BufferAllocation {
    pub fn as_slice(&self) -> wgpu::BufferSlice {
        self.buffer.slice(self.offset..self.offset + self.size)
    }
}

/// For allocating gpu buffer space to write into
/// This allows using one buffer for multiple uniform/storage data writes which is more efficient
pub struct DynamicBufferAllocator {
    buffer_spaces: Vec<DynamicBufferSpace>,
    descriptor: wgpu::BufferDescriptor<'static>,
    alignment: u64,
    max_size: u64,
}

impl DynamicBufferAllocator {
    pub fn new(wgpu: &WgpuContext, usage: wgpu::BufferUsages) -> Self {
        let alignment = match usage {
            wgpu::BufferUsages::UNIFORM => wgpu.limits.min_uniform_buffer_offset_alignment as u64,
            wgpu::BufferUsages::STORAGE => wgpu.limits.min_storage_buffer_offset_alignment as u64,
            _ => wgpu::COPY_BUFFER_ALIGNMENT,
        };

        let descriptor = wgpu::BufferDescriptor {
            label: None,
            size: 1024 * 8, // 8kb starting size
            usage: usage | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };

        let max_size = match usage {
            wgpu::BufferUsages::UNIFORM => wgpu.limits.max_uniform_buffer_binding_size as u64,
            wgpu::BufferUsages::STORAGE => wgpu.limits.max_storage_buffer_binding_size as u64,
            _ => wgpu.limits.max_buffer_size,
        };

        Self {
            buffer_spaces: Vec::new(),
            alignment,
            descriptor,
            max_size,
        }
    }

    /// Finds a space inside one of the buffers where size fits and writes to that space with data
    pub fn alloc_write<T: bytemuck::Pod>(
        &mut self,
        wgpu: &WgpuContext,
        data: &[T],
    ) -> BufferAllocation {
        let size = std::mem::size_of_val(data) as u64;

        // Aligns size to alignment since UBOs require a min alignment for dynamic indexing
        let size_aligned = size.next_multiple_of(self.alignment);
        assert!(
            size_aligned <= self.max_size,
            "requested size ({0}) is greater than max size ({1})",
            size_aligned,
            self.max_size
        );

        // Find space where size fits
        for space in &mut self.buffer_spaces {
            if size_aligned <= space.buffer.size() - space.cursor {
                let offset = space.cursor;
                space.cursor += size_aligned;

                wgpu.queue
                    .write_buffer(&space.buffer, offset, bytemuck::cast_slice(data));

                return BufferAllocation {
                    offset,
                    size,
                    buffer: RefId::clone(&space.buffer),
                };
            }
        }

        // Didn't find any so grow and try again
        self.grow(wgpu, size_aligned);
        self.alloc_write(wgpu, data)
    }

    pub fn grow(&mut self, wgpu: &WgpuContext, required_size: u64) {
        let size_aligned = required_size.next_multiple_of(self.alignment);
        self.descriptor.size = u64::clamp(self.descriptor.size * 2, size_aligned, self.max_size);
        self.buffer_spaces
            .push(DynamicBufferSpace::new(wgpu, &self.descriptor));
    }

    pub fn free(&mut self) {
        for space in &mut self.buffer_spaces {
            space.cursor = 0;
        }
    }
}
