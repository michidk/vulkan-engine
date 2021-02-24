use std::ops::Deref;

use ash::vk;

#[allow(dead_code)]
pub struct BufferWrapper {
    pub buffer: vk::Buffer,
    allocation: vk_mem::Allocation,
    allocation_info: vk_mem::AllocationInfo,
    capacity: u64,
    size: u64,
    buffer_usage: vk::BufferUsageFlags,
    memory_usage: vk_mem::MemoryUsage,
}

impl BufferWrapper {
    pub fn new(
        allocator: &vk_mem::Allocator,
        capacity: u64,
        buffer_usage: vk::BufferUsageFlags,
        memory_usage: vk_mem::MemoryUsage,
    ) -> Result<Self, vk_mem::error::Error> {
        let allocation_create_info = vk_mem::AllocationCreateInfo {
            usage: memory_usage,
            ..Default::default()
        };

        let (buffer, allocation, allocation_info) = allocator.create_buffer(
            &vk::BufferCreateInfo::builder()
                .size(capacity)
                .usage(buffer_usage)
                .build(),
            &allocation_create_info,
        )?;

        Ok(Self {
            buffer,
            allocation,
            allocation_info,
            capacity,
            size: 0,
            buffer_usage,
            memory_usage,
        })
    }

    pub fn fill<T: Sized>(
        &mut self,
        allocator: &vk_mem::Allocator,
        data: &[T],
    ) -> Result<(), vk_mem::error::Error> {
        let bytes_to_write = (data.len() * std::mem::size_of::<T>()) as u64;
        if bytes_to_write > self.capacity {
            log::warn!("Not enough memory allocated in buffer; Resizing");
            self.resize(allocator, bytes_to_write)?;
        }

        let data_ptr = allocator.map_memory(&self.allocation)? as *mut T;
        unsafe {
            data_ptr.copy_from_nonoverlapping(data.as_ptr(), data.len());
        };
        allocator.unmap_memory(&self.allocation);
        self.size = bytes_to_write;
        Ok(())
    }

    pub fn get_size(&self) -> u64 {
        self.size
    }

    fn resize(
        &mut self,
        allocator: &vk_mem::Allocator,
        new_capacity: u64,
    ) -> Result<(), vk_mem::error::Error> {
        allocator.destroy_buffer(self.buffer, &self.allocation);
        let new_buffer = BufferWrapper::new(
            allocator,
            new_capacity,
            self.buffer_usage,
            self.memory_usage,
        )?;
        *self = new_buffer;
        Ok(())
    }

    pub fn cleanup(&mut self, allocator: &vk_mem::Allocator) {
        allocator.destroy_buffer(self.buffer, &self.allocation)
    }
}

impl Deref for BufferWrapper {
    type Target = vk::Buffer;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}
