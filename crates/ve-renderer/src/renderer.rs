//! This module contains the main [`Renderer`] code.
//!
//! See the [`Renderer`] docs.

use std::{
    ffi::{c_void, CStr, CString},
    fmt::Debug,
};

use ash::{extensions::khr, vk};
use raw_window_handle::RawDisplayHandle;

use crate::{error::CreationError, version::Version};

const MAX_FRAMES_IN_FLIGHT: usize = 3;

/// Holds information passed directly to the Vulkan API.
///
/// This is in practice probably used for game-specific driver optimizations.
pub struct AppInfo<'a> {
    /// The name of the application
    pub name: &'a str,
    /// The version of the application
    pub version: Version,
}

/// Contains the most important state needed for rendering.
pub struct Renderer {
    pub(crate) device: ash::Device,
    pub(crate) instance: ash::Instance,
    pub(crate) entry: ash::Entry,

    pub(crate) khr_surface: khr::Surface,
    pub(crate) khr_swapchain: khr::Swapchain,

    pub(crate) device_info: DeviceInfo,

    pub(crate) graphics_queue: vk::Queue,
    pub(crate) transfer_queue: Option<vk::Queue>,

    pub(crate) command_pools: Vec<vk::CommandPool>,
}

impl Debug for Renderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Renderer")
            .field("device", &"...")
            .field("instance", &"...")
            .field("entry", &"...")
            .field("khr_surface", &"...")
            .field("khr_swapchain", &"...")
            .field("device_info", &self.device_info)
            .field("graphics_queue", &self.graphics_queue)
            .field("transfer_queue", &self.transfer_queue)
            .finish()
    }
}

#[derive(Debug)]
pub(crate) struct DeviceInfo {
    pub(crate) physical_device: vk::PhysicalDevice,

    pub(crate) graphics_queue_family: u32,
    pub(crate) transfer_queue_family: Option<u32>,
}

const ENGINE_NAME: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"ve_renderer\0") };
const ENGINE_VERSION: Version = Version::new(0, 1, 0);

impl Renderer {
    /// Initializes a new [`Renderer`].
    ///
    /// # Errors
    /// - [`CreationError::LoadingError`] when ash initialization fails
    /// - [`CreationError::UnsupportedInstanceVersion`] when the driver does not support Vulkan 1.2 instances
    /// - [`CreationError::MissingInstanceExtension`] when a required instance extension is not supported
    /// - [`CreationError::NoDevice`] when no suitable GPU was found
    /// - [`CreationError::Vk`] when a Vulkan API function returns an unexpected error
    pub fn new(
        app_info: &AppInfo,
        display_handle: RawDisplayHandle,
    ) -> Result<Self, CreationError> {
        log::info!("Initializing Vulkan");
        let entry = unsafe { ash::Entry::load()? };

        let instance_version = Self::get_instance_version(&entry)?;
        log::info!("Vulkan instance version: {}", instance_version);
        if !instance_version.compatible_with(Version::VK12) {
            log::error!("Vulkan instance version is not compatible with Vulkan 1.2");
            return Err(CreationError::UnsupportedInstanceVersion(instance_version));
        }

        let instance = Self::create_instance(&entry, display_handle, app_info)?;

        let (device, device_info) = Self::create_device(&entry, &instance, display_handle)?;

        let graphics_queue =
            unsafe { device.get_device_queue(device_info.graphics_queue_family, 0) };
        let transfer_queue = unsafe {
            device_info
                .transfer_queue_family
                .map(|tf| device.get_device_queue(tf, 0))
        };

        let khr_surface = khr::Surface::new(&entry, &instance);
        let khr_swapchain = khr::Swapchain::new(&instance, &device);

        let mut command_pools = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            let create_info = vk::CommandPoolCreateInfo::builder()
                .queue_family_index(device_info.graphics_queue_family);
            unsafe {
                command_pools.push(device.create_command_pool(&create_info, None)?);
            }
        }

        Ok(Self {
            device,
            instance,
            entry,
            device_info,

            khr_surface,
            khr_swapchain,

            graphics_queue,
            transfer_queue,

            command_pools,
        })
    }

    fn get_instance_version(entry: &ash::Entry) -> Result<Version, vk::Result> {
        let instance_version = entry.try_enumerate_instance_version()?;
        Ok(instance_version.map_or(Version::VK10, Version::from_vk_version))
    }

    fn select_instance_extensions(
        entry: &ash::Entry,
        display_handle: RawDisplayHandle,
    ) -> Result<Vec<*const i8>, CreationError> {
        let required_extensions = ash_window::enumerate_required_extensions(display_handle)?;
        log::debug!("Required instance extensions are: {:#?}", required_extensions.iter().map(|ptr| unsafe{ CStr::from_ptr(*ptr).to_str().unwrap() } ).collect::<Vec<_>>());

        let extensions = entry.enumerate_instance_extension_properties(None)?;

        // Check that all extensions required for windowing are supported
        for req in required_extensions {
            let req_name = unsafe { CStr::from_ptr(*req) };

            if !extensions
                .iter()
                .any(|e| unsafe { CStr::from_ptr(e.extension_name.as_ptr()) } == req_name)
            {
                log::error!("Required instance extension {} is not supported", req_name.to_str().unwrap());
                return Err(CreationError::MissingInstanceExtension(
                    req_name
                        .to_str()
                        .expect("Invalid extension name")
                        .to_string(),
                ));
            }
        }

        let res = required_extensions.to_vec();

        // Check for optional extensions goes here

        log::info!("Enabling instance extensions: {:#?}", res.iter().map(|ptr| unsafe{ CStr::from_ptr(*ptr).to_str().unwrap() }).collect::<Vec<_>>());
        Ok(res)
    }

    fn create_instance(
        entry: &ash::Entry,
        display_handle: RawDisplayHandle,
        app_info: &AppInfo,
    ) -> Result<ash::Instance, CreationError> {
        let extensions = Self::select_instance_extensions(entry, display_handle)?;

        let app_name = CString::new(app_info.name).expect("Invalid application name");

        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .application_version(app_info.version.into_vk_version())
            .engine_name(ENGINE_NAME)
            .engine_version(ENGINE_VERSION.into_vk_version())
            .api_version(vk::API_VERSION_1_3);

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extensions);

        unsafe { Ok(entry.create_instance(&create_info, None)?) }
    }

    fn check_physical_device(
        instance: &ash::Instance,
        device: vk::PhysicalDevice,
    ) -> Result<bool, vk::Result> {
        let props = unsafe { instance.get_physical_device_properties(device) };

        let device_name = unsafe { CStr::from_ptr(props.device_name.as_ptr()).to_str().unwrap() };
        log::info!("Checking device {}", device_name);

        // Check that the device supports VK 1.2
        let version = Version::from_vk_version(props.api_version);
        log::info!("Device Vulkan version: {}", version);
        if !version.compatible_with(Version::VK12) {
            log::info!("Device does not support Vulkan 1.2");
            return Ok(false);
        }

        let extensions = unsafe { instance.enumerate_device_extension_properties(device)? };

        // Check that VK_KHR_swapchain is supported
        if !extensions.iter().any(
            |ext| unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) } == khr::Swapchain::name(),
        ) {
            log::info!("Device does not support VK_KHR_swapchain");
            return Ok(false);
        }

        let mut features12 = vk::PhysicalDeviceVulkan12Features::default();
        let mut features = vk::PhysicalDeviceFeatures2::builder().push_next(&mut features12);
        unsafe {
            // Safe because we already checked that the device supports VK 1.2
            instance.get_physical_device_features2(device, &mut features);
        }

        if features12.imageless_framebuffer != vk::TRUE {
            log::info!("Device does not support imagelessFramebuffer");
            return Ok(false);
        }

        Ok(true)
    }

    fn check_present_support(
        entry: &ash::Entry,
        instance: &ash::Instance,
        device: vk::PhysicalDevice,
        queue: u32,
        display_handle: RawDisplayHandle,
    ) -> bool {
        match display_handle {
            RawDisplayHandle::Windows(_) => {
                let ext = khr::Win32Surface::new(entry, instance);
                unsafe { ext.get_physical_device_win32_presentation_support(device, queue) }
            }
            RawDisplayHandle::Wayland(handle) => {
                let ext = khr::WaylandSurface::new(entry, instance);
                unsafe {
                    ext.get_physical_device_wayland_presentation_support(
                        device,
                        queue,
                        &mut *handle.display,
                    )
                }
            }
            RawDisplayHandle::Xlib(handle) => {
                let ext = khr::XlibSurface::new(entry, instance);
                unsafe {
                    let ptr = handle.display as *mut *const c_void;
                    ext.get_physical_device_xlib_presentation_support(
                        device,
                        queue,
                        &mut *ptr,
                        handle.screen as u32,
                    )
                }
            }
            RawDisplayHandle::Xcb(handle) => {
                let ext = khr::XcbSurface::new(entry, instance);
                unsafe {
                    ext.get_physical_device_xcb_presentation_support(
                        device,
                        queue,
                        &mut *handle.connection,
                        handle.screen as u32,
                    )
                }
            }
            RawDisplayHandle::Android(_) => {
                // On Android, every queue can present
                true
            }
            RawDisplayHandle::AppKit(_) | RawDisplayHandle::UiKit(_) => {
                // On iOS and macOS, every queue can present
                true
            }
            // We should not have come this far, as ash_window should have reported an error earlier
            _ => unreachable!(),
        }
    }

    fn find_queues(
        entry: &ash::Entry,
        instance: &ash::Instance,
        device: vk::PhysicalDevice,
        display_handle: RawDisplayHandle,
    ) -> Result<(Option<u32>, Option<u32>), vk::Result> {
        let queues = unsafe { instance.get_physical_device_queue_family_properties(device) };

        let mut gfx_family = None;
        for (index, qf) in queues.iter().enumerate() {
            if qf.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                && Self::check_present_support(
                    entry,
                    instance,
                    device,
                    index as u32,
                    display_handle,
                )
            {
                gfx_family = Some(index as u32);
                break;
            }
        }
        let Some(gfx_family) = gfx_family else { 
            log::info!("Device has no graphics queue");
            return Ok((None, None)); 
        };
        log::info!("Device has graphics family: {}", gfx_family);

        let transfer_family = queues.iter().enumerate().find_map(|(index, qf)| {
            if index != gfx_family as usize
                && !qf
                    .queue_flags
                    .intersects(vk::QueueFlags::GRAPHICS | vk::QueueFlags::COMPUTE)
                && qf.queue_flags.contains(vk::QueueFlags::TRANSFER)
            {
                log::info!("Device has async transfer queue family: {}", index);
                Some(index as u32)
            } else {
                None
            }
        });

        Ok((Some(gfx_family), transfer_family))
    }

    fn select_physical_device(
        entry: &ash::Entry,
        instance: &ash::Instance,
        display_handle: RawDisplayHandle,
    ) -> Result<DeviceInfo, CreationError> {
        let devices = unsafe { instance.enumerate_physical_devices()? };

        for dev in devices {
            if !Self::check_physical_device(instance, dev)? {
                continue;
            }

            let (Some(gfx_queue), transfer_queue) =
                Self::find_queues(entry, instance, dev, display_handle)? else {
                    continue;
                };

            return Ok(DeviceInfo {
                physical_device: dev,
                graphics_queue_family: gfx_queue,
                transfer_queue_family: transfer_queue,
            });
        }

        Err(CreationError::NoDevice)
    }

    fn create_device(
        entry: &ash::Entry,
        instance: &ash::Instance,
        display_handle: RawDisplayHandle,
    ) -> Result<(ash::Device, DeviceInfo), CreationError> {
        let device_info = Self::select_physical_device(entry, instance, display_handle)?;

        let prio = [1.0];
        let mut queue_infos = Vec::with_capacity(2);
        queue_infos.push(
            vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(device_info.graphics_queue_family)
                .queue_priorities(&prio)
                .build(),
        );
        if let Some(tq) = device_info.transfer_queue_family {
            queue_infos.push(
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(tq)
                    .queue_priorities(&prio)
                    .build(),
            );
        }

        let extensions = [khr::Swapchain::name().as_ptr()];

        let mut features12 =
            vk::PhysicalDeviceVulkan12Features::builder().imageless_framebuffer(true);

        let create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_infos)
            .enabled_extension_names(&extensions)
            .push_next(&mut features12);

        unsafe {
            let device = instance.create_device(device_info.physical_device, &create_info, None)?;

            Ok((device, device_info))
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        log::info!("Destroying renderer");
    }
}
