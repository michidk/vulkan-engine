use std::{collections::BTreeMap, ffi::CStr};

use ash::vk;

use super::error::{GraphicsError, GraphicsResult};

const VULKAN_VERSION: (u32, u32) = (1, 2);

pub fn select_physical_device(
    instance: &ash::Instance,
) -> GraphicsResult<(
    vk::PhysicalDevice,
    vk::PhysicalDeviceProperties,
    vk::PhysicalDeviceFeatures,
    bool,
)> {
    let phys_devs = unsafe { instance.enumerate_physical_devices() }?;
    let mut candidates: BTreeMap<
        u32,
        (
            vk::PhysicalDevice,
            vk::PhysicalDeviceProperties,
            vk::PhysicalDeviceFeatures,
            bool,
        ),
    > = BTreeMap::new();

    for device in phys_devs {
        let properties = unsafe { instance.get_physical_device_properties(device) };
        let features = unsafe { instance.get_physical_device_features(device) };

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
            log::info!("GPU detected: {}", name);

            if ext_memory_budget_supported {
                log::info!("    supports VK_EXT_memory_budget");
            }
        }

        if vk::api_version_major(properties.api_version) != VULKAN_VERSION.0
            || vk::api_version_minor(properties.api_version) < VULKAN_VERSION.1
        {
            continue;
        }

        let mut score: u32 = 0;

        // prefere discrete gpu
        if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
            score += 1000;
        }

        // possible texture size affects graphics quality
        score += properties.limits.max_image_dimension2_d;

        // require geometry shader
        if features.geometry_shader == vk::FALSE {
            continue;
        }

        candidates.insert(
            score,
            (device, properties, features, ext_memory_budget_supported),
        );
    }

    if candidates.is_empty() {
        return Err(GraphicsError::NoSuitableGpu);
    }

    Ok(*candidates.values().last().unwrap())
}
