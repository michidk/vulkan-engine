use std::ffi::CStr;

use ash::vk;

use super::{
    error::{GraphicsError, GraphicsResult},
    RendererConfig,
};

const VULKAN_VERSION: (u32, u32) = (1, 0);

pub(crate) fn get_candidates(
    instance: &ash::Instance,
) -> GraphicsResult<Vec<(vk::PhysicalDevice, vk::PhysicalDeviceProperties, bool)>> {
    let phys_devs = unsafe { instance.enumerate_physical_devices() }?;
    let mut candidates = Vec::new();

    for device in phys_devs {
        let properties = unsafe { instance.get_physical_device_properties(device) };

        let extensions = unsafe { instance.enumerate_device_extension_properties(device)? };

        let ext_memory_budget_supported = extensions.iter().any(|ext| {
            let c_str = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) };
            c_str.to_str().unwrap() == "VK_EXT_memory_budget"
        });

        #[cfg(debug_assertions)]
        {
            let name = String::from(
                unsafe { CStr::from_ptr(properties.device_name.as_ptr()) }
                    .to_str()
                    .unwrap(),
            );
            log::info!(
                "GPU detected: {} ({:08X}:{:08X})",
                name,
                properties.vendor_id,
                properties.device_id
            );

            if ext_memory_budget_supported {
                log::info!("    supports VK_EXT_memory_budget");
            }
        }

        if vk::api_version_major(properties.api_version) != VULKAN_VERSION.0
            || vk::api_version_minor(properties.api_version) < VULKAN_VERSION.1
        {
            continue;
        }

        candidates.push((device, properties, ext_memory_budget_supported));
    }

    Ok(candidates)
}

pub(crate) fn select_physical_device(
    instance: &ash::Instance,
    config: Option<&RendererConfig>,
) -> GraphicsResult<(vk::PhysicalDevice, vk::PhysicalDeviceProperties, bool)> {
    let override_ids = config
        .map(|cfg| cfg.gpu_vendor_id.zip(cfg.gpu_device_id))
        .unwrap_or(None);

    let candidates = get_candidates(instance)?;

    if candidates.is_empty() {
        return Err(GraphicsError::NoSuitableGpu);
    }

    if let Some((vendor, device)) = override_ids {
        let dev = candidates
            .iter()
            .find(|d| d.1.vendor_id == vendor && d.1.device_id == device);

        if let Some(dev) = dev {
            return Ok(*dev);
        } else {
            log::warn!("Override GPU with id {:08X}:{:08X} not found or not suitable, using default device", vendor, device);
        }
    }

    Ok(*candidates.first().unwrap())
}
