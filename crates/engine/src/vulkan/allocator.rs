use std::cell::RefCell;

use ash::vk;
use gpu_allocator::{vulkan::AllocationCreateDesc, AllocatorDebugSettings};

use super::error::GraphicsResult;

pub struct Allocator {
    alloc: RefCell<gpu_allocator::vulkan::Allocator>,
    device: ash::Device,
}

impl Allocator {
    pub fn new(
        instance: ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: ash::Device,
    ) -> Self {
        let debug = cfg!(debug_assertions);

        let alloc =
            gpu_allocator::vulkan::Allocator::new(&gpu_allocator::vulkan::AllocatorCreateDesc {
                instance,
                device: device.clone(),
                physical_device,
                debug_settings: AllocatorDebugSettings {
                    log_memory_information: debug,
                    log_leaks_on_shutdown: debug,
                    store_stack_traces: false,
                    log_allocations: debug,
                    log_frees: debug,
                    log_stack_traces: false,
                },
                buffer_device_address: false,
            })
            .unwrap();

        Self {
            alloc: alloc.into(),
            device,
        }
    }

    pub fn create_buffer(
        &self,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        location: gpu_allocator::MemoryLocation,
    ) -> GraphicsResult<(vk::Buffer, gpu_allocator::vulkan::Allocation)> {
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .build();

        let buffer = unsafe { self.device.create_buffer(&buffer_info, None) }?;
        let requirements = unsafe { self.device.get_buffer_memory_requirements(buffer) };

        let alloc = self.alloc.borrow_mut().allocate(&AllocationCreateDesc {
            name: "Buffer allocation",
            requirements,
            location,
            linear: true,
        })?;

        unsafe {
            self.device
                .bind_buffer_memory(buffer, alloc.memory(), alloc.offset())?;
        }

        Ok((buffer, alloc))
    }

    pub fn destroy_buffer(&self, buffer: vk::Buffer, alloc: gpu_allocator::vulkan::Allocation) {
        unsafe {
            self.device.destroy_buffer(buffer, None);
        }
        self.alloc.borrow_mut().free(alloc).unwrap();
    }

    pub fn create_image(
        &self,
        width: u32,
        height: u32,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
        location: gpu_allocator::MemoryLocation,
    ) -> GraphicsResult<(vk::Image, gpu_allocator::vulkan::Allocation)> {
        let image_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(format)
            .extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .build();

        let image = unsafe { self.device.create_image(&image_info, None) }?;
        let requirements = unsafe { self.device.get_image_memory_requirements(image) };

        let alloc = self.alloc.borrow_mut().allocate(&AllocationCreateDesc {
            name: "Image allocation",
            requirements,
            location,
            linear: false,
        })?;

        unsafe {
            self.device
                .bind_image_memory(image, alloc.memory(), alloc.offset())?;
        }

        Ok((image, alloc))
    }

    pub fn destroy_image(&self, image: vk::Image, alloc: gpu_allocator::vulkan::Allocation) {
        unsafe {
            self.device.destroy_image(image, None);
        }
        self.alloc.borrow_mut().free(alloc).unwrap();
    }

    pub fn free(&self, alloc: gpu_allocator::vulkan::Allocation) {
        self.alloc.borrow_mut().free(alloc).unwrap();
    }
}
