use std::collections::BTreeMap;

use ash::{version::InstanceV1_0, vk};

use super::error::{GraphicsError, GraphicsResult};

const VULKAN_VERSION: (u32, u32) = (1, 2);

pub fn select_physical_device(
    instance: &ash::Instance,
) -> GraphicsResult<(
        vk::PhysicalDevice,
        vk::PhysicalDeviceProperties,
        vk::PhysicalDeviceFeatures,
    )>
{
    let phys_devs = unsafe { instance.enumerate_physical_devices() }?;
    let mut candidates: BTreeMap<
        u32,
        (
            vk::PhysicalDevice,
            vk::PhysicalDeviceProperties,
            vk::PhysicalDeviceFeatures,
        ),
    > = BTreeMap::new();

    for device in phys_devs {
        let properties = unsafe { instance.get_physical_device_properties(device) };
        let features = unsafe { instance.get_physical_device_features(device) };

        #[cfg(debug_assertions)]
        {
            use std::ffi::CStr;

            let name = String::from(
                unsafe { CStr::from_ptr(properties.device_name.as_ptr()) }
                    .to_str()
                    .unwrap(),
            );
            log::info!("GPU detected: {}", name);
        }

        if vk::version_major(properties.api_version) != VULKAN_VERSION.0
            || vk::version_minor(properties.api_version) < VULKAN_VERSION.1
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

        candidates.insert(score, (device, properties, features));
    }

    if candidates.is_empty() {
        return Err(GraphicsError::NoSuitableGpu);
    }

    Ok(candidates.pop_last().unwrap().1) // use physical device with the highest score
}
