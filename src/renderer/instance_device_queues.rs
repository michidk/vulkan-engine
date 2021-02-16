use std::{collections::BTreeMap, ffi::CString};

use ash::{
    extensions::{ext::DebugUtils, khr},
    version::{DeviceV1_0, EntryV1_0, InstanceV1_0},
    vk,
};

use super::{debug, surface, RendererError, DEFAULT_WINDOW_INFO};

pub fn init_instance(
    window: &winit::window::Window,
    entry: &ash::Entry,
) -> Result<ash::Instance, ash::InstanceError> {
    let app_name = CString::new(DEFAULT_WINDOW_INFO.title).unwrap();

    // // https://hoj-senna.github.io/ashen-engine/text/002_Beginnings.html
    let app_info = vk::ApplicationInfo::builder()
        .application_name(&app_name)
        .application_version(vk::make_version(0, 0, 1))
        .engine_name(&app_name)
        .engine_version(vk::make_version(0, 42, 0))
        .api_version(vk::make_version(1, 0, 106));

    // sooo, we need to use display extensions as well
    // let extension_name_pointers: Vec<*const i8> =
    //     vec![ash::extensions::ext::DebugUtils::name().as_ptr()];
    // but let's do it the cool way
    // https://hoj-senna.github.io/ashen-engine/text/006_Window.html

    let surface_extensions = ash_window::enumerate_required_extensions(window).unwrap();
    let mut extension_names_raw = surface_extensions
        .iter()
        .map(|ext| ext.as_ptr())
        .collect::<Vec<_>>();
    extension_names_raw.push(DebugUtils::name().as_ptr()); // still wanna use the debug extensions

    let mut instance_create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_extension_names(&extension_names_raw);

    // handle validation layers
    let startup_debug_severity = debug::startup_debug_severity();
    let startup_debug_type = debug::startup_debug_type();
    let debug_create_info =
        &mut debug::get_debug_create_info(startup_debug_severity, startup_debug_type);

    let layer_names = debug::get_layer_names();
    if debug::ENABLE_VALIDATION_LAYERS && debug::has_validation_layers_support(&entry) {
        instance_create_info = instance_create_info
            .push_next(debug_create_info)
            .enabled_layer_names(&layer_names);
    }

    unsafe { entry.create_instance(&instance_create_info, None) }
}

// choose gpu
// https://hoj-senna.github.io/ashen-engine/text/004_Physical_device.html
// https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Physical_devices_and_queue_families
pub fn init_physical_device_and_properties(
    instance: &ash::Instance,
) -> Result<
    (
        vk::PhysicalDevice,
        vk::PhysicalDeviceProperties,
        vk::PhysicalDeviceFeatures,
    ),
    RendererError,
> {
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

        let mut score: u32 = 0;

        // prefere discrete gpu
        if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
            score += 1000;
        }

        // possible texture size affects graphics quality
        score += properties.limits.max_image_dimension2_d;

        // require geometry shader
        if features.geometry_shader == vk::FALSE {
            score = 0;
        }

        candidates.insert(score, (device, properties, features));

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
    }

    if candidates.is_empty() {
        return Err(RendererError::NoSuitableGpu);
    }

    Ok(candidates.pop_first().unwrap().1)
}

pub struct QueueFamilies {
    pub graphics_q_index: u32,
    pub present_q_index: u32,
}

impl QueueFamilies {
    pub fn init(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surfaces: &surface::SurfaceWrapper,
    ) -> Result<QueueFamilies, RendererError> {
        let queues =
            QueueFamilies::find_suiltable_queue_family(instance, physical_device, surfaces)?;
        Ok(QueueFamilies {
            graphics_q_index: queues.0,
            present_q_index: queues.1,
        })
    }

    fn find_suiltable_queue_family(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surfaces: &surface::SurfaceWrapper,
    ) -> Result<(u32, u32), RendererError> {
        let queuefamilyproperties =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let mut found_graphics_q_index = None;
        let mut found_present_q_index = None;
        for (index, qfam) in queuefamilyproperties.iter().enumerate() {
            if qfam.queue_count > 0 {
                if qfam.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                    found_graphics_q_index = Some(index as u32);
                }

                if surfaces.get_physical_device_surface_support(physical_device, index)? {
                    found_present_q_index = Some(index as u32);
                }
            }

            if found_graphics_q_index.is_some() && found_present_q_index.is_some() {
                break;
            }
        }

        match found_graphics_q_index.zip(found_present_q_index) {
            Some(zipped) => Ok(zipped),
            _ => Err(RendererError::NoSuitableQueueFamily),
        }
    }
}

pub struct Queues {
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
}

pub fn init_device_and_queues(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    queue_families: &QueueFamilies,
) -> Result<(ash::Device, Queues), vk::Result> {
    let device_extension_names_raw = [khr::Swapchain::name().as_ptr()];
    // https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkPhysicalDeviceFeatures.html
    // required for wireframe fill mode
    let features = vk::PhysicalDeviceFeatures::builder().fill_mode_non_solid(true);
    let priorities = [1.0];

    let queue_info = [
        vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_families.graphics_q_index)
            .queue_priorities(&priorities)
            .build(),
        vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_families.present_q_index)
            .queue_priorities(&priorities)
            .build(),
    ];

    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_info)
        .enabled_extension_names(&device_extension_names_raw)
        .enabled_features(&features);

    let logical_device: ash::Device =
        unsafe { instance.create_device(physical_device, &device_create_info, None) }?;

    let graphics_queue =
        unsafe { logical_device.get_device_queue(queue_families.graphics_q_index as u32, 0) };

    let present_queue =
        unsafe { logical_device.get_device_queue(queue_families.present_q_index as u32, 0) };

    Ok((
        logical_device,
        Queues {
            graphics_queue,
            present_queue,
        },
    ))
}
