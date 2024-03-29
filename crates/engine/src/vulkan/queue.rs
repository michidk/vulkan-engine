use std::ffi::CString;

use ash::{extensions::khr, vk};

use super::{
    error::{GraphicsError, GraphicsResult},
    surface,
};

pub struct PoolsWrapper {
    pub commandpool_graphics: vk::CommandPool,
}

impl PoolsWrapper {
    pub fn init(
        logical_device: &ash::Device,
        queue_families: &QueueFamilies,
    ) -> GraphicsResult<Self> {
        let graphics_commandpool_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(queue_families.graphics_q_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        let commandpool_graphics =
            unsafe { logical_device.create_command_pool(&graphics_commandpool_info, None) }?;
        Ok(Self {
            commandpool_graphics,
        })
    }

    pub fn cleanup(&self, logical_device: &ash::Device) {
        unsafe {
            logical_device.destroy_command_pool(self.commandpool_graphics, None);
        }
    }
}

pub fn create_commandbuffers(
    logical_device: &ash::Device,
    pools: &PoolsWrapper,
    amount: usize,
) -> GraphicsResult<Vec<vk::CommandBuffer>> {
    let commandbuf_allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(pools.commandpool_graphics)
        .command_buffer_count(amount as u32);
    unsafe { Ok(logical_device.allocate_command_buffers(&commandbuf_allocate_info)?) }
}

pub struct QueueFamilies {
    pub graphics_q_index: u32,
}

impl QueueFamilies {
    pub fn init(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surface: &surface::SurfaceWrapper,
    ) -> GraphicsResult<QueueFamilies> {
        let queues = QueueFamilies::find_suitable_queue_family(instance, physical_device, surface)?;
        Ok(QueueFamilies {
            graphics_q_index: queues,
        })
    }

    fn find_suitable_queue_family(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surface: &surface::SurfaceWrapper,
    ) -> GraphicsResult<u32> {
        let queuefamilyproperties =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let mut found_graphics_q_index = None;
        for (index, qfam) in queuefamilyproperties.iter().enumerate() {
            let surface_support =
                surface.get_physical_device_surface_support(physical_device, index)?;

            if qfam.queue_flags.contains(vk::QueueFlags::GRAPHICS) && surface_support {
                // found perfect queue family, break
                found_graphics_q_index = Some(index as u32);
            }
        }

        if let Some(index) = found_graphics_q_index {
            Ok(index)
        } else {
            Err(GraphicsError::NoSuitableQueueFamily)
        }
    }
}

pub struct Queues {
    pub graphics_queue: vk::Queue,
}

pub fn init_device_and_queues(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    queue_families: &QueueFamilies,
    ext_memory_budget: bool,
) -> GraphicsResult<(ash::Device, Queues)> {
    let ext_memory_budget_name = CString::new("VK_EXT_memory_budget").unwrap();

    let device_extension_names_raw = [
        khr::Swapchain::name().as_ptr(),
        ext_memory_budget_name.as_ptr(),
    ];
    // https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkPhysicalDeviceFeatures.html
    // required for wireframe fill mode
    let features = vk::PhysicalDeviceFeatures::builder().fill_mode_non_solid(true); // TODO: check if feature is supported before force-enabling it
    let priorities = [1.0];

    let queue_info = [vk::DeviceQueueCreateInfo::builder()
        .queue_family_index(queue_families.graphics_q_index)
        .queue_priorities(&priorities)
        .build()];

    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_info)
        .enabled_extension_names(if ext_memory_budget {
            &device_extension_names_raw
        } else {
            &device_extension_names_raw[0..=0]
        })
        .enabled_features(&features);

    let logical_device: ash::Device =
        unsafe { instance.create_device(physical_device, &device_create_info, None) }?;

    let graphics_queue =
        unsafe { logical_device.get_device_queue(queue_families.graphics_q_index, 0) };

    Ok((logical_device, Queues { graphics_queue }))
}
