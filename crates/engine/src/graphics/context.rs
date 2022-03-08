use std::ffi::{CStr, CString};

use ash::{vk, extensions::khr};

use crate::{AppInfo, ENGINE_NAME, ENGINE_VERSION};

use super::error::{GraphicsError, GraphicsResult};

pub(crate) const MAX_FRAMES_IN_FLIGHT: usize = 3;

pub(crate) struct Context {
    pub(crate) entry: ash::Entry,
    pub(crate) instance: ash::Instance,
    pub(crate) device: ash::Device,

    pub(crate) khr_surface: khr::Surface,
    pub(crate) khr_swapchain: khr::Swapchain,

    pub(crate) physical_device: vk::PhysicalDevice,
    graphics_queue: QueueInfo,
    transfer_queue: Option<QueueInfo>,

    current_frame: usize,

    graphics_command_pool: vk::CommandPool,
    graphics_command_buffers: Vec<vk::CommandBuffer>,
    transfer_command_pool: Option<vk::CommandPool>,
    transfer_command_buffers: Option<Vec<vk::CommandBuffer>>,
}

#[derive(Debug, Clone, Copy)]
struct QueueInfo {
    family_index: u32,
    index: u32,
    queue: vk::Queue,
}

impl Context {
    pub(crate) fn new(app_info: &AppInfo) -> GraphicsResult<Self> {
        let entry = unsafe{ash::Entry::load()}.map_err(|_| GraphicsError::VulkanUnavailable)?;
        let instance = Self::create_instance(&entry, app_info)?;

        let physical_devices = Self::get_suitable_devices(&instance)?;
        if physical_devices.is_empty() {
            return Err(GraphicsError::NoDevice);
        }

        let physical_device = physical_devices.first().unwrap();
        log::info!("Using physical device {}", physical_device.name);

        let (device, graphics_queue, transfer_queue) = Self::create_device(&instance, physical_device)?;

        let khr_surface = khr::Surface::new(&entry, &instance);
        let khr_swapchain = khr::Swapchain::new(&instance, &device);

        let (graphics_command_pool, graphics_command_buffers) = {
            let info = vk::CommandPoolCreateInfo::builder()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER | vk::CommandPoolCreateFlags::TRANSIENT)
                .queue_family_index(graphics_queue.family_index);
            let pool = unsafe {device.create_command_pool(&info, None)?};

            let info = vk::CommandBufferAllocateInfo::builder()
                .command_pool(pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(MAX_FRAMES_IN_FLIGHT as u32);
            let buffers = unsafe{device.allocate_command_buffers(&info)?};

            (pool, buffers)
        };

        let (transfer_command_pool, transfer_command_buffers) = if let Some(transfer_queue) = transfer_queue {
            let info = vk::CommandPoolCreateInfo::builder()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER | vk::CommandPoolCreateFlags::TRANSIENT)
                .queue_family_index(transfer_queue.family_index);
            let pool = unsafe {device.create_command_pool(&info, None)?};

            let info = vk::CommandBufferAllocateInfo::builder()
                .command_pool(pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(MAX_FRAMES_IN_FLIGHT as u32);
            let buffers = unsafe{device.allocate_command_buffers(&info)?};

            (Some(pool), Some(buffers))
        } else {
            (None, None)
        };

        Ok(Self {
            entry,
            instance,
            device,
            khr_surface,
            khr_swapchain,
            physical_device: physical_device.physical_device,
            graphics_queue,
            transfer_queue,

            current_frame: 0,

            graphics_command_pool,
            graphics_command_buffers,
            transfer_command_pool,
            transfer_command_buffers,
        })
    }

    pub(crate) fn device_wait_idle(&self) {
        unsafe {
            self.device.device_wait_idle().expect("Failed to wait_idle()");
        }
    }

    pub(crate) fn new_frame(&mut self) -> GraphicsResult<()> {
        self.current_frame += 1;
        self.current_frame %= MAX_FRAMES_IN_FLIGHT;

        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe {
            self.device.begin_command_buffer(self.graphics_command_buffers[self.current_frame], &begin_info)?;
        }

        if let Some(buffers) = &self.transfer_command_buffers {
            let begin_info = vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            unsafe {
                self.device.begin_command_buffer(buffers[self.current_frame], &begin_info)?;
            }
        }

        Ok(())
    }
    pub(crate) fn end_frame(&mut self) -> GraphicsResult<()> {
        unsafe {
            self.device.end_command_buffer(self.graphics_command_buffers[self.current_frame])?;
        }

        if let Some(buffers) = &self.transfer_command_buffers {
            unsafe {
                self.device.end_command_buffer(buffers[self.current_frame])?;
            }
        }

        Ok(())
    }

    pub(crate) fn submit(&mut self) -> GraphicsResult<()> {
        
    }

    pub(crate) fn graphics_command_buffer(&self) -> vk::CommandBuffer {
        self.graphics_command_buffers[self.current_frame]
    }
    pub(crate) fn transfer_command_buffer(&self) -> Option<vk::CommandBuffer> {
        if let Some(buffers) = &self.transfer_command_buffers {
            Some(buffers[self.current_frame])
        } else {
            None
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_command_pool(self.graphics_command_pool, None);
            if let Some(transfer_command_pool) = self.transfer_command_pool {
                self.device.destroy_command_pool(transfer_command_pool, None);
            }

            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}

impl Context {
    fn surface_extension_candidates() -> Vec<&'static CStr> {
        #[cfg(windows)]
        return vec![
            khr::Win32Surface::name(),
        ];

        #[cfg(linux)]
        return vec![
            khr::XlibSurface::name(),
            khr::XcbSurface::name(),
            khr::WaylandSurface::name(),
        ];

        #[cfg(not(any(windows, linux)))]
        compile_error!("Unsupported platform");
    }

    fn check_instance_extensions(entry: &ash::Entry) -> GraphicsResult<Vec<*const i8>> {
        let mut res = vec![khr::Surface::name().as_ptr()];

        let exts = entry.enumerate_instance_extension_properties()?;

        // check that VK_KHR_surface is supported
        if !exts.iter().any(|ext| unsafe {
            CStr::from_ptr(ext.extension_name.as_ptr()) == khr::Surface::name()
        }) {
            return Err(GraphicsError::SurfaceNotSupported);
        }

        // add every supported platform-dependent surface extension to the list
        let candidates = Self::surface_extension_candidates();
        for cand in candidates {
            if exts.iter().any(|ext| unsafe {
                CStr::from_ptr(ext.extension_name.as_ptr()) == khr::Surface::name()
            }) {
                res.push(cand.as_ptr());
            }
        }

        // if res is not at least 2 long, no platform dependent extension is supported
        if res.len() < 2 {
            Err(GraphicsError::SurfaceNotSupported)
        } else {
            Ok(res)
        }
    }

    fn create_instance(entry: &ash::Entry, app_info: &AppInfo) -> GraphicsResult<ash::Instance> {
        let extensions = Self::check_instance_extensions(entry)?;
        for ext in &extensions {
            log::info!("Enabling instance extension {}", unsafe{CStr::from_ptr(*ext).to_str().unwrap()});
        }

        let app_name = CString::new(app_info.app_name).unwrap();
        let engine_name = CString::new(ENGINE_NAME).unwrap();

        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .application_version(vk::make_api_version(0, app_info.app_version.major(), app_info.app_version.minor(), app_info.app_version.patch()))
            .engine_name(&engine_name)
            .engine_version(vk::make_api_version(0, ENGINE_VERSION.major(), ENGINE_VERSION.minor(), ENGINE_VERSION.patch()))
            .api_version(vk::API_VERSION_1_2);

        let instance_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extensions);

        unsafe{
            Ok(entry.create_instance(&instance_info, None)?)
        }
    }

    fn get_suitable_devices(instance: &ash::Instance) -> GraphicsResult<Vec<PhysicalDeviceInfo>> {
        let mut res = Vec::new();

        let devs = unsafe{instance.enumerate_physical_devices()?};
        for dev in devs {
            let mut props = vk::PhysicalDeviceProperties2::builder();
            let mut features12 = vk::PhysicalDeviceVulkan12Features::builder();
            let mut features = vk::PhysicalDeviceFeatures2::builder()
                .push_next(&mut features12);
            unsafe {
                instance.get_physical_device_properties2(dev, &mut props);
                instance.get_physical_device_features2(dev, &mut features);
            }
            let queue_families = unsafe{instance.get_physical_device_queue_family_properties(dev)};
            let extensions = unsafe{instance.enumerate_device_extension_properties(dev)?};

            let dev_name = unsafe{CStr::from_ptr(props.properties.device_name.as_ptr()).to_str().unwrap()};
            log::info!("Detected Vulkan Device {} ({}.{}.{})",
                dev_name,
                vk::api_version_major(props.properties.api_version),
                vk::api_version_minor(props.properties.api_version),
                vk::api_version_patch(props.properties.api_version)
            );

            // check vulkan version is compatible with vulkan 1.2
            if vk::api_version_major(props.properties.api_version) != 1
                || vk::api_version_minor(props.properties.api_version) < 2
            {
                log::info!("Device Vulkan version not compatible with Vulkan 1.2");
                continue;
            }

            // check device supports VK_KHR_swapchain
            if !extensions.iter().any(|e| unsafe {
                CStr::from_ptr(e.extension_name.as_ptr()) == khr::Swapchain::name()
            }) {
                log::info!("Device does not support VK_KHR_swapchain");
                continue;
            }

            // check device supports imageless framebuffers
            if features12.imageless_framebuffer != vk::TRUE {
                log::info!("Device does not support imageless framebuffers");
                continue;
            }

            let graphics_family = {
                let gf = queue_families.iter().enumerate().find(|(_, qf)| {
                    qf.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                });

                if let Some((i, _)) = gf {
                    log::info!("Device queue family {} is suitable for graphics", i);
                    i as u32
                } else {
                    log::info!("Device has no suitable graphics queue");
                    continue
                }
            };

            let transfer_family = {
                let tf = queue_families.iter().enumerate().find(|(i, qf)| {
                    *i as u32 != graphics_family && qf.queue_flags.contains(vk::QueueFlags::TRANSFER) && !qf.queue_flags.contains(vk::QueueFlags::GRAPHICS) && !qf.queue_flags.contains(vk::QueueFlags::COMPUTE)
                });

                if let Some((i, _)) = tf {
                    log::info!("Device queue family {} is suitable for async transfer", i);
                    Some(i as u32)
                } else {
                    log::info!("Device has no async transfer family");
                    None
                }
            };

            log::info!("Device is suitable");
            res.push(PhysicalDeviceInfo {
                physical_device: dev,
                graphics_family,
                transfer_family,
                name: dev_name.to_string(),
            });
        }

        Ok(res)
    }

    fn create_device(instance: &ash::Instance, info: &PhysicalDeviceInfo) -> GraphicsResult<(ash::Device, QueueInfo, Option<QueueInfo>)> {
        let prio = [1.0];

        let mut queue_infos = vec![
            vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(info.graphics_family)
                .queue_priorities(&prio)
                .build()
        ];
        if let Some(tf) = info.transfer_family {
            queue_infos.push(vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(tf)
                .queue_priorities(&prio)
                .build()
            );
        }

        let extensions = [
            khr::Swapchain::name().as_ptr()
        ];

        let mut features12 = vk::PhysicalDeviceVulkan12Features::builder()
            .imageless_framebuffer(true);

        let dev_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_infos)
            .enabled_extension_names(&extensions)
            .push_next(&mut features12);

        let device = unsafe{instance.create_device(info.physical_device, &dev_info, None)?};

        let gfx_queue = QueueInfo {
            family_index: info.graphics_family,
            index: 0,
            queue: unsafe{device.get_device_queue(info.graphics_family, 0)},
        };

        let transfer_queue = info.transfer_family.map(|tf| QueueInfo {
            family_index: tf,
            index: 0,
            queue: unsafe{device.get_device_queue(tf, 0)},
        });

        Ok((
            device,
            gfx_queue,
            transfer_queue,
        ))
    }
}

struct PhysicalDeviceInfo {
    physical_device: vk::PhysicalDevice,
    graphics_family: u32,
    transfer_family: Option<u32>,
    name: String,
}
