use std::{mem, ops::Deref};

use ash::vk;

pub trait VulkanBuffer {
    fn get_size(&self) -> u64;
    fn get_buffer(&self) -> vk::Buffer;
    fn get_offset(&self, current_frame_index: u8) -> vk::DeviceSize;
}

pub trait MutableBuffer<T: Sized>: VulkanBuffer {
    fn set_data(
        &mut self,
        allocator: &vk_mem::Allocator,
        data: &T,
        current_frame_index: u8,
    ) -> Result<(), vk_mem::error::Error>;
}

pub trait ResizableBuffer<T>: MutableBuffer<T> {
    fn resize(new_size: u64) -> Result<(), vk_mem::error::Error>;
}

pub struct PerFrameUniformBuffer<T: Sized> {
    buffer: vk::Buffer,
    allocation: vk_mem::Allocation,
    data_size: u64,
    aligned_data_size: u64,
    mapping: *mut T,
}

impl<T: Sized> PerFrameUniformBuffer<T> {
    pub fn new(
        phys_props: &vk::PhysicalDeviceProperties,
        allocator: &vk_mem::Allocator,
        num_frames: u64,
        buffer_usage: vk::BufferUsageFlags,
    ) -> Result<Self, vk_mem::error::Error> {
        let alignment = phys_props.limits.min_uniform_buffer_offset_alignment;
        let data_size = mem::size_of::<T>() as u64;
        let aligned_data_size = (data_size + alignment - 1) / alignment * alignment;

        let buffer_info = vk::BufferCreateInfo::builder()
            .size(aligned_data_size * num_frames)
            .usage(buffer_usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .build();
        let alloc_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::CpuToGpu,
            ..Default::default()
        };

        let (buffer, allocation, _) = allocator.create_buffer(&buffer_info, &alloc_info)?;

        let mapping = allocator.map_memory(&allocation)? as *mut T;

        Ok(Self {
            buffer,
            allocation,
            data_size,
            aligned_data_size,
            mapping,
        })
    }

    pub fn destroy(&self, allocator: &vk_mem::Allocator) {
        allocator.unmap_memory(&self.allocation);
        allocator.destroy_buffer(self.buffer, &self.allocation);
    }
}

impl<T: Sized> VulkanBuffer for PerFrameUniformBuffer<T> {
    fn get_size(&self) -> u64 {
        self.data_size
    }

    fn get_buffer(&self) -> vk::Buffer {
        self.buffer
    }

    fn get_offset(&self, current_frame_index: u8) -> vk::DeviceSize {
        self.aligned_data_size * current_frame_index as u64
    }
}

impl<T: Sized> MutableBuffer<T> for PerFrameUniformBuffer<T> {
    fn set_data(
        &mut self,
        _: &vk_mem::Allocator,
        data: &T,
        current_frame_index: u8,
    ) -> Result<(), vk_mem::Error> {
        let offset = current_frame_index as u64 * self.aligned_data_size;

        unsafe {
            let ptr = (self.mapping as *mut u8).offset(offset as isize) as *mut T;
            ptr.copy_from_nonoverlapping(data, 1);
        }

        Ok(())
    }
}

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

#[allow(dead_code)]
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
