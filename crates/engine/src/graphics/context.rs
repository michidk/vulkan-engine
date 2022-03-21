use std::{ffi::{CStr, CString}, cell::RefCell, mem::ManuallyDrop};

use ash::{vk, extensions::khr};

use crate::{AppInfo, ENGINE_NAME, ENGINE_VERSION};

use super::{error::{GraphicsError, GraphicsResult}, renderer::Renderer, window::Window};

pub(crate) struct Context {
    pub(crate) entry: ash::Entry,
    pub(crate) instance: ash::Instance,
    pub(crate) device: ash::Device,

    pub(crate) khr_surface: khr::Surface,
    pub(crate) khr_swapchain: khr::Swapchain,

    pub(crate) physical_device: vk::PhysicalDevice,
    pub(crate) graphics_queue: QueueInfo,
    pub(crate) transfer_queue: Option<QueueInfo>,

    pub(crate) frame_counter: usize,
    pub(crate) max_frames_in_flight: usize,

    allocator: ManuallyDrop<RefCell<gpu_allocator::vulkan::Allocator>>,

    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,

    fences: Vec<vk::Fence>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct QueueInfo {
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

        let allocator = ManuallyDrop::new(RefCell::new(gpu_allocator::vulkan::Allocator::new(&gpu_allocator::vulkan::AllocatorCreateDesc {
            instance: instance.clone(),
            device: device.clone(),
            physical_device: physical_device.physical_device,
            debug_settings: gpu_allocator::AllocatorDebugSettings { 
                log_memory_information: true, 
                log_leaks_on_shutdown: false, 
                store_stack_traces: false, 
                log_allocations: false, 
                log_frees: false, 
                log_stack_traces: false, 
            },
            buffer_device_address: false,
        })?));

        let command_pool = {
            let info = vk::CommandPoolCreateInfo::builder()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .queue_family_index(graphics_queue.family_index);
            unsafe {
                device.create_command_pool(&info, None)?
            }
        };
        let command_buffers = {
            let alloc_info = vk::CommandBufferAllocateInfo::builder()
                .command_pool(command_pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(3);
            unsafe {
                device.allocate_command_buffers(&alloc_info)?
            }
        };

        let mut fences = Vec::with_capacity(3);
        for _ in 0..3 {
            let info = vk::FenceCreateInfo::builder()
                .flags(vk::FenceCreateFlags::SIGNALED);
            let fence = unsafe{device.create_fence(&info, None)?};
            fences.push(fence);
        }

        Ok(Self {
            entry,
            instance,
            device,
            khr_surface,
            khr_swapchain,
            physical_device: physical_device.physical_device,
            graphics_queue,
            transfer_queue,
            
            frame_counter: 0,
            max_frames_in_flight: 3,

            allocator,

            command_pool,
            command_buffers,

            fences,
        })
    }

    pub(crate) fn device_wait_idle(&self) {
        unsafe {
            self.device.device_wait_idle().expect("Failed to wait_idle()");
        }
    }

    pub(crate) fn create_image(&self, width: u32, height: u32, format: vk::Format, usage: vk::ImageUsageFlags, debug_name: &str) -> GraphicsResult<(gpu_allocator::vulkan::Allocation, vk::Image)> {
        let view_formats = [format];
        let mut format_list = vk::ImageFormatListCreateInfo::builder()
            .view_formats(&view_formats);
        
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
            .push_next(&mut format_list);
        let image = unsafe{self.device.create_image(&image_info, None)?};
        let requirements = unsafe{self.device.get_image_memory_requirements(image)};
        
        let alloc_info = gpu_allocator::vulkan::AllocationCreateDesc {
            name: debug_name,
            requirements,
            location: gpu_allocator::MemoryLocation::GpuOnly,
            linear: false,
        };
        let alloc = self.allocator.borrow_mut().allocate(&alloc_info)?;

        unsafe {
            self.device.bind_image_memory(image, alloc.memory(), alloc.offset())?;
        }

        Ok((alloc, image))
    }

    pub(crate) fn destroy_image(&self, alloc: gpu_allocator::vulkan::Allocation, image: vk::Image) {
        self.allocator.borrow_mut().free(alloc).expect("Failed to free image allocation");
        unsafe {
            self.device.destroy_image(image, None);
        }
    }

    pub(crate) fn render_frame<R: Renderer>(&self, renderer: &mut R, window: &mut Window) -> GraphicsResult<()> {
        unsafe {
            self.device.wait_for_fences(&[self.fences[self.frame_counter % self.max_frames_in_flight]], true, u64::MAX)?;
            self.device.reset_fences(&[self.fences[self.frame_counter % self.max_frames_in_flight]])?;
        }

        let mut should_resize = false;

        let image_index;
        loop {
            let res = unsafe{self.khr_swapchain.acquire_next_image(window.swapchain.handle, u64::MAX, window.acquire_semaphores[self.frame_counter % self.max_frames_in_flight], vk::Fence::null())};
            match res {
                Ok((i, suboptimal)) => {
                    if suboptimal {
                        should_resize = true;
                    }

                    image_index = i;
                    break;
                },
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    self.device_wait_idle();
                    window.recreate_swapchain()?;
                    renderer.set_size((window.swapchain.size.width, window.swapchain.size.height))?;
                },
                Err(e) => return Err(GraphicsError::Vk(e)),
            }
        }

        let command_buffer = self.command_buffers[self.frame_counter % self.max_frames_in_flight];

        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe {
            self.device.begin_command_buffer(command_buffer, &begin_info)?;
        }

        let render_image = renderer.render_frame(command_buffer)?;
        let (render_width, render_height) = renderer.get_size();

        unsafe {
            self.device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::BY_REGION,
                &[],
                &[],
                &[
                    vk::ImageMemoryBarrier::builder()
                        .src_access_mask(vk::AccessFlags::empty())
                        .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                        .old_layout(vk::ImageLayout::UNDEFINED)
                        .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                        .src_queue_family_index(0)
                        .dst_queue_family_index(0)
                        .image(window.swapchain.images[image_index as usize])
                        .subresource_range(vk::ImageSubresourceRange {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            base_mip_level: 0,
                            level_count: 1,
                            base_array_layer: 0,
                            layer_count: 1,
                        })
                        .build(),
                ]
            );

            self.device.cmd_blit_image(
                command_buffer,
                render_image,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                window.swapchain.images[image_index as usize],
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[
                    vk::ImageBlit {
                        src_subresource: vk::ImageSubresourceLayers {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            mip_level: 0,
                            base_array_layer: 0,
                            layer_count: 1,
                        },
                        src_offsets: [
                            vk::Offset3D {
                                x: 0,
                                y: 0,
                                z: 0,
                            },
                            vk::Offset3D {
                                x: render_width as i32,
                                y: render_height as i32,
                                z: 1,
                            },
                        ],
                        dst_subresource: vk::ImageSubresourceLayers {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            mip_level: 0,
                            base_array_layer: 0,
                            layer_count: 1,
                        },
                        dst_offsets: [
                            vk::Offset3D {
                                x: 0,
                                y: 0,
                                z: 0,
                            },
                            vk::Offset3D {
                                x: window.swapchain.size.width as i32 - 20,
                                y: window.swapchain.size.height as i32 - 20,
                                z: 1,
                            },
                        ],
                    }
                ],
                vk::Filter::LINEAR,
            );

            self.device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                vk::DependencyFlags::BY_REGION,
                &[],
                &[],
                &[
                    vk::ImageMemoryBarrier::builder()
                        .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                        .dst_access_mask(vk::AccessFlags::empty())
                        .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                        .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                        .src_queue_family_index(0)
                        .dst_queue_family_index(0)
                        .image(window.swapchain.images[image_index as usize])
                        .subresource_range(vk::ImageSubresourceRange {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            base_mip_level: 0,
                            level_count: 1,
                            base_array_layer: 0,
                            layer_count: 1,
                        })
                        .build(),
                ]
            );

            self.device.end_command_buffer(command_buffer)?;
        }

        let wait_semaphores = [window.acquire_semaphores[self.frame_counter % self.max_frames_in_flight]];
        let wait_stages = [vk::PipelineStageFlags::TRANSFER];
        let signal_semaphores = [window.render_semaphores[self.frame_counter % self.max_frames_in_flight]];
        let command_buffers = [command_buffer];
        let submits = vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores);
        let res = unsafe {
            self.device.queue_submit(self.graphics_queue.queue, &[submits.build()], self.fences[self.frame_counter % self.max_frames_in_flight])?;

            let swapchains = [window.swapchain.handle];
            let indices = [image_index];
            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&swapchains)
                .image_indices(&indices);

            self.khr_swapchain.queue_present(self.graphics_queue.queue, &present_info)
        };
        match res {
            Ok(suboptimal) => {
                if suboptimal {
                    should_resize = true;
                }
            },
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                should_resize = true;
            },
            Err(e) => return Err(GraphicsError::Vk(e)),
        }

        if should_resize {
            self.device_wait_idle();
            window.recreate_swapchain()?;
            renderer.set_size((window.swapchain.size.width, window.swapchain.size.height))?;
        }

        Ok(())
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.allocator);

            for fence in &self.fences {
                self.device.destroy_fence(*fence, None);
            }

            self.device.destroy_command_pool(self.command_pool, None);

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
