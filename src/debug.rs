use ash::{extensions::ext, version::EntryV1_0, vk};
use std::{ffi::c_void, os::raw::c_char};
use std::{
    ffi::{CStr, CString},
    lazy::SyncLazy,
};

#[cfg(debug_assertions)]
pub const ENABLE_VALIDATION_LAYERS: bool = true;
#[cfg(not(debug_assertions))]
pub const ENABLE_VALIDATION_LAYERS: bool = false;

//const VALIDATION_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];
pub static VALIDATION_LAYERS: SyncLazy<[CString; 1]> =
    SyncLazy::new(|| [CString::new("VK_LAYER_KHRONOS_validation").unwrap()]);

pub fn startup_debug_severity() -> vk::DebugUtilsMessageSeverityFlagsEXT {
    vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
}

pub fn startup_debug_type() -> vk::DebugUtilsMessageTypeFlagsEXT {
    vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
        | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
        | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
}

pub fn debug_severity() -> vk::DebugUtilsMessageSeverityFlagsEXT {
    vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
        | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
        | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
        | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
}

pub fn debug_type() -> vk::DebugUtilsMessageTypeFlagsEXT {
    vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
        | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
        | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
}

// make sure you dont discard the layer_names or memory will be lost
pub fn get_layer_names() -> Vec<*const c_char> {
    VALIDATION_LAYERS
        .iter()
        .map(|name| name.as_ptr())
        .collect::<Vec<_>>()
}

pub fn get_debug_create_info<'a>(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    types: vk::DebugUtilsMessageTypeFlagsEXT,
) -> vk::DebugUtilsMessengerCreateInfoEXTBuilder<'a> {
    vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(severity)
        .message_type(types)
        .pfn_user_callback(Some(vulkan_debug_utils_callback))
}

pub fn has_validation_layers_support(entry: &ash::Entry) -> bool {
    for required in VALIDATION_LAYERS.iter() {
        let found = entry
            .enumerate_instance_layer_properties()
            .unwrap()
            .iter()
            .any(|layer| {
                let name = unsafe { CStr::from_ptr(layer.layer_name.as_ptr()) };
                required.as_c_str() == name
            });

        if !found {
            log::error!("Validation layers are enabled but are not installed on the system!");
            return false;
        }
    }

    true
}

unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> vk::Bool32 {
    let message = CStr::from_ptr((*p_callback_data).p_message);
    let severity = format!("{:?}", message_severity).to_lowercase();
    let ty = format!("{:?}", message_type).to_lowercase();

    match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => log::error!("[{}] {:?}", ty, message),
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => log::warn!("[{}] {:?}", ty, message),
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => log::info!("[{}] {:?}", ty, message),
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => log::trace!("[{}] {:?}", ty, message),
        _ => log::error!("Unknown severity ({}; message: {:?})", severity, message),
    };

    vk::FALSE
}

pub struct DebugMessenger {
    loader: ext::DebugUtils,
    messenger: vk::DebugUtilsMessengerEXT,
}

impl DebugMessenger {
    pub fn init(
        entry: &ash::Entry,
        instance: &ash::Instance,
    ) -> Result<DebugMessenger, vk::Result> {
        let loader = ext::DebugUtils::new(entry, instance);
        let messenger = unsafe {
            loader.create_debug_utils_messenger(
                &get_debug_create_info(debug_severity(), debug_type()),
                None,
            )?
        };

        Ok(DebugMessenger { loader, messenger })
    }
}

impl Drop for DebugMessenger {
    fn drop(&mut self) {
        unsafe {
            self.loader
                .destroy_debug_utils_messenger(self.messenger, None)
        };
    }
}
