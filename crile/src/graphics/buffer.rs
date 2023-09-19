use crate::{RefId, WGPUContext};

struct DynamicBufferSpace {
    buffer: RefId<wgpu::Buffer>,
    cursor: u64,
}

impl DynamicBufferSpace {
    fn new(wgpu: &WGPUContext, descriptor: &wgpu::BufferDescriptor) -> Self {
        Self {
            buffer: wgpu.device.create_buffer(&descriptor).into(),
            cursor: 0,
        }
    }
}

pub struct BufferAllocation {
    pub offset: u64,
    pub buffer: RefId<wgpu::Buffer>,
}

// For allocating gpu buffer space to write into
// This allows using one buffer for multiple uniform/storage data writes which is more efficient
pub struct DynamicBufferAllocator {
    buffer_spaces: Vec<DynamicBufferSpace>,
    descriptor: wgpu::BufferDescriptor<'static>,
    alignment: u64,
}

impl DynamicBufferAllocator {
    pub fn new(
        wgpu: &WGPUContext,
        alignment: u64,
        descriptor: wgpu::BufferDescriptor<'static>,
    ) -> Self {
        Self {
            buffer_spaces: vec![DynamicBufferSpace::new(wgpu, &descriptor)],
            alignment,
            descriptor,
        }
    }

    pub fn allocate(&mut self, wgpu: &WGPUContext, size: u64) -> BufferAllocation {
        // TODO: use size.div_ceil once https://github.com/rust-lang/rust/issues/88581 is stablized
        // Aligns size to alignment
        let size = (size / self.alignment + 1) * self.alignment;
        assert!(
            size <= self.descriptor.size,
            "requested size is greater than buffer size"
        );

        // Find space where size fits
        for space in &mut self.buffer_spaces {
            if size <= self.descriptor.size - space.cursor {
                let offset = space.cursor;
                space.cursor += size;
                return BufferAllocation {
                    offset,
                    buffer: RefId::clone(&space.buffer),
                };
            }
        }

        // Didn't find any so grow and try again
        self.buffer_spaces
            .push(DynamicBufferSpace::new(wgpu, &self.descriptor));
        self.allocate(wgpu, size)
    }

    pub fn free(&mut self) {
        for space in &mut self.buffer_spaces {
            space.cursor = 0;
        }
    }
}
