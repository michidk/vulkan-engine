use std::{mem, ops::Deref};

use ash::vk;
use gpu_allocator::{MemoryLocation, SubAllocation};

use super::allocator::Allocator;

pub trait VulkanBuffer {
    fn get_size(&self) -> u64;
    fn get_buffer(&self) -> vk::Buffer;
    fn get_offset(&self, current_frame_index: u8) -> vk::DeviceSize;
}

pub trait MutableBuffer<T: Sized>: VulkanBuffer {
    fn set_data(
        &mut self,
        allocator: &Allocator,
        data: &T,
        current_frame_index: u8,
    );
}

pub trait ResizableBuffer<T>: MutableBuffer<T> {
    fn resize(new_size: u64);
}

pub struct PerFrameUniformBuffer<T: Sized> {
    buffer: vk::Buffer,
    allocation: SubAllocation,
    data_size: u64,
    aligned_data_size: u64,
    mapping: *mut T,
}

impl<T: Sized> PerFrameUniformBuffer<T> {
    pub fn new(
        phys_props: &vk::PhysicalDeviceProperties,
        allocator: &Allocator,
        num_frames: u64,
        buffer_usage: vk::BufferUsageFlags,
    ) -> Self {
        let alignment = phys_props.limits.min_uniform_buffer_offset_alignment;
        let data_size = mem::size_of::<T>() as u64;
        let aligned_data_size = (data_size + alignment - 1) / alignment * alignment;

        let (buffer, allocation) = allocator.create_buffer(aligned_data_size * num_frames, buffer_usage, gpu_allocator::MemoryLocation::CpuToGpu);

        let mapping = Allocator::get_ptr(&allocation) as *mut T;

        Self {
            buffer,
            allocation,
            data_size,
            aligned_data_size,
            mapping,
        }
    }

    pub fn destroy(&self, allocator: &Allocator) {
        allocator.free(&self.allocation);
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
        _: &Allocator,
        data: &T,
        current_frame_index: u8,
    ) {
        let offset = current_frame_index as u64 * self.aligned_data_size;

        unsafe {
            let ptr = (self.mapping as *mut u8).offset(offset as isize) as *mut T;
            ptr.copy_from_nonoverlapping(data, 1);
        }
    }
}

#[allow(dead_code)]
pub struct BufferWrapper {
    pub buffer: vk::Buffer,
    allocation: SubAllocation,
    capacity: u64,
    size: u64,
    buffer_usage: vk::BufferUsageFlags,
    memory_usage: MemoryLocation,
}

#[allow(dead_code)]
impl BufferWrapper {
    pub fn new(
        allocator: &Allocator,
        capacity: u64,
        buffer_usage: vk::BufferUsageFlags,
        memory_usage: MemoryLocation,
    ) -> Self {
        let (buffer, allocation) = allocator.create_buffer(capacity, buffer_usage, memory_usage);

        Self {
            buffer,
            allocation,
            capacity,
            size: 0,
            buffer_usage,
            memory_usage,
        }
    }

    pub fn fill<T: Sized>(
        &mut self,
        allocator: &Allocator,
        data: &[T],
    ) {
        let bytes_to_write = (data.len() * std::mem::size_of::<T>()) as u64;
        if bytes_to_write > self.capacity {
            log::warn!("Not enough memory allocated in buffer; Resizing");
            self.resize(allocator, bytes_to_write);
        }

        let data_ptr = Allocator::get_ptr(&self.allocation) as *mut T;
        unsafe {
            data_ptr.copy_from_nonoverlapping(data.as_ptr(), data.len());
        };
        self.size = bytes_to_write;
    }

    pub fn get_size(&self) -> u64 {
        self.size
    }

    fn resize(
        &mut self,
        allocator: &Allocator,
        new_capacity: u64,
    ) {
        allocator.destroy_buffer(self.buffer, &self.allocation);
        let new_buffer = BufferWrapper::new(
            allocator,
            new_capacity,
            self.buffer_usage,
            self.memory_usage,
        );
        *self = new_buffer;
    }

    pub fn cleanup(&mut self, allocator: &Allocator) {
        allocator.destroy_buffer(self.buffer, &self.allocation)
    }
}

impl Deref for BufferWrapper {
    type Target = vk::Buffer;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}
