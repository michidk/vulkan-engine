use std::{cell::RefCell, collections::HashMap, ffi::c_void, rc::Rc};

use ash::vk;
use gpu_allocator::{AllocationCreateDesc, AllocatorDebugSettings, MemoryLocation, VulkanAllocator, VulkanAllocatorCreateDesc};

pub use gpu_allocator::SubAllocation;

pub struct Allocator {
    alloc: RefCell<VulkanAllocator>,
    device: Rc<ash::Device>,
}

impl Allocator {
    pub fn new(instance: ash::Instance, device: Rc<ash::Device>, physical_device: vk::PhysicalDevice, buffer_device_address: bool) -> Self {
        let alloc = VulkanAllocator::new(&VulkanAllocatorCreateDesc {
            instance,
            device: (*device).clone(),
            physical_device,
            debug_settings: AllocatorDebugSettings {
                log_memory_information: true,
                log_leaks_on_shutdown: true,
                store_stack_traces: false,
                log_allocations: false,
                log_frees: false,
                log_stack_traces: false,
            },
            buffer_device_address,
        });

        Self {
            alloc: alloc.into(),
            device,
        }
    }

    pub fn create_buffer(&self, size: vk::DeviceSize, usage: vk::BufferUsageFlags, location: MemoryLocation) -> (vk::Buffer, SubAllocation) {
        let buffer = unsafe{self.device.create_buffer(
            &vk::BufferCreateInfo::builder()
                .size(size)
                .usage(usage)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .build(),
            None
        )}.unwrap();

        let req = unsafe{self.device.get_buffer_memory_requirements(buffer)};

        let alloc = self.alloc.borrow_mut().allocate(&AllocationCreateDesc {
            name: "buffer",
            requirements: req,
            location,
            linear: true,
        }).unwrap();

        unsafe {
            self.device.bind_buffer_memory(buffer, alloc.memory(), alloc.offset());
        }

        (buffer, alloc)
    }

    pub fn destroy_buffer(&self, buffer: vk::Buffer, alloc: &SubAllocation) {
        self.alloc.borrow_mut().free(alloc.clone());
        unsafe {
            self.device.destroy_buffer(buffer, None);
        }
    }

    pub fn create_image(&self, width: u32, height: u32, usage: vk::ImageUsageFlags, format: vk::Format) -> (vk::Image, SubAllocation) {
        let image = unsafe{self.device.create_image(
            &vk::ImageCreateInfo::builder()
                .image_type(vk::ImageType::TYPE_2D)
                .format(format)
                .extent(vk::Extent3D{ width, height, depth: 1 })
                .mip_levels(1)
                .array_layers(1)
                .samples(vk::SampleCountFlags::TYPE_1)
                .tiling(vk::ImageTiling::OPTIMAL)
                .usage(usage)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .build(),
            None
        )}.unwrap();

        let req = unsafe{self.device.get_image_memory_requirements(image)};

        let alloc = self.alloc.borrow_mut().allocate(&AllocationCreateDesc {
            name: "image",
            requirements: req,
            location: MemoryLocation::GpuOnly,
            linear: false
        }).unwrap();

        unsafe{
            self.device.bind_image_memory(image, alloc.memory(), alloc.offset());
        }

        (image, alloc)
    }

    pub fn destroy_image(&self, image: vk::Image, alloc: &SubAllocation) {
        self.alloc.borrow_mut().free(alloc.clone());
        unsafe {
            self.device.destroy_image(image, None);
        }
    }

    pub fn free(&self, alloc: &SubAllocation) {
        self.alloc.borrow_mut().free(alloc.clone());
    }

    pub fn get_ptr(alloc: &SubAllocation) -> *mut c_void {
        alloc.mapped_ptr().unwrap().as_ptr()
    }

}
