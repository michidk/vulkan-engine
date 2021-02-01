use std::ffi::{c_void, CStr, CString};

use ash::{
    extensions::{ext::DebugUtils, khr},
    version::{DeviceV1_0, EntryV1_0, InstanceV1_0},
    vk,
};

use winit::{
    error::OsError,
    event_loop::{EventLoop, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
};

#[derive(Debug, Clone, PartialEq)]
struct AppInfo {
    width: u32,
    height: u32,
    title: &'static str,
}

impl AppInfo {
    fn into_window<T: 'static>(
        self,
        window_target: &EventLoopWindowTarget<T>,
    ) -> Result<Window, OsError> {
        WindowBuilder::new()
            .with_title(self.title)
            .with_inner_size(winit::dpi::LogicalSize::new(
                f64::from(self.width),
                f64::from(self.height),
            ))
            .build(window_target)
    }
}

const DEFAULT_WINDOW_INFO: AppInfo = AppInfo {
    width: 1920,
    height: 1080,
    title: "VulkanTriangle",
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // setting up logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    // https://hoj-senna.github.io/ashen-engine/text/008_Cleanup.html
    let eventloop = EventLoop::new();
    let window = DEFAULT_WINDOW_INFO.clone().into_window(&eventloop).unwrap();
    let mut engine = Engine::init(window)?;
    use winit::event::{Event, WindowEvent};
    eventloop.run(move |event, _, controlflow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *controlflow = winit::event_loop::ControlFlow::Exit;
        }
        Event::MainEventsCleared => {
            // doing the work here (later)
            engine.window.request_redraw();
        }
        Event::RedrawRequested(_) => {
            let (image_index, _) = unsafe {
                engine
                    .swapchain
                    .swapchain_loader
                    .acquire_next_image(
                        engine.swapchain.swapchain,
                        std::u64::MAX,
                        engine.swapchain.image_available[engine.swapchain.current_image],
                        vk::Fence::null(),
                    )
                    .expect("image acquisition trouble")
            };
            unsafe {
                engine
                    .device
                    .wait_for_fences(
                        &[engine.swapchain.may_begin_drawing[engine.swapchain.current_image]],
                        true,
                        std::u64::MAX,
                    )
                    .expect("fence-waiting");
                engine
                    .device
                    .reset_fences(&[
                        engine.swapchain.may_begin_drawing[engine.swapchain.current_image]
                    ])
                    .expect("resetting fences");
            }
            let semaphores_available =
                [engine.swapchain.image_available[engine.swapchain.current_image]];
            let waiting_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let semaphores_finished =
                [engine.swapchain.rendering_finished[engine.swapchain.current_image]];
            let commandbuffers = [engine.commandbuffers[image_index as usize]];
            let submit_info = [vk::SubmitInfo::builder()
                .wait_semaphores(&semaphores_available)
                .wait_dst_stage_mask(&waiting_stages)
                .command_buffers(&commandbuffers)
                .signal_semaphores(&semaphores_finished)
                .build()];
            unsafe {
                engine
                    .device
                    .queue_submit(
                        engine.queues.graphics_queue,
                        &submit_info,
                        engine.swapchain.may_begin_drawing[engine.swapchain.current_image],
                    )
                    .expect("queue submission");
            };
            let swapchains = [engine.swapchain.swapchain];
            let indices = [image_index];
            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(&semaphores_finished)
                .swapchains(&swapchains)
                .image_indices(&indices);
            unsafe {
                engine
                    .swapchain
                    .swapchain_loader
                    .queue_present(engine.queues.graphics_queue, &present_info)
                    .expect("queue presentation");
            };
            engine.swapchain.current_image =
                (engine.swapchain.current_image + 1) % engine.swapchain.amount_of_images as usize;
        }
        _ => {}
    });
}

fn init_instance(window: &Window, entry: &ash::Entry) -> Result<ash::Instance, ash::InstanceError> {
    let app_name = CString::new(DEFAULT_WINDOW_INFO.title).unwrap();
    // // https://hoj-senna.github.io/ashen-engine/text/002_Beginnings.html
    let app_info = vk::ApplicationInfo::builder()
        .application_name(&app_name)
        .application_version(vk::make_version(0, 0, 1))
        .engine_name(&app_name)
        .engine_version(vk::make_version(0, 42, 0))
        .api_version(vk::make_version(1, 0, 106));

    // debug stuff
    // https://hoj-senna.github.io/ashen-engine/text/003_Validation_layers.html
    let layer_names = [CString::new("VK_LAYER_KHRONOS_validation").unwrap()];
    let layers_names_raw: Vec<*const i8> = layer_names
        .iter()
        .map(|raw_name| raw_name.as_ptr())
        .collect();

    // debug config during creation
    let mut debugcreateinfo = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(
            // vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
            //     | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE |
            // vk::DebugUtilsMessageSeverityFlagsEXT::INFO |
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        )
        .pfn_user_callback(Some(vulkan_debug_utils_callback));

    // sooo, we need to use display extensions as well
    // let extension_name_pointers: Vec<*const i8> =
    //     vec![ash::extensions::ext::DebugUtils::name().as_ptr()];
    // so lets do it the cool way
    // https://hoj-senna.github.io/ashen-engine/text/006_Window.html

    let surface_extensions = ash_window::enumerate_required_extensions(window).unwrap();
    let mut extension_names_raw = surface_extensions
        .iter()
        .map(|ext| ext.as_ptr())
        .collect::<Vec<_>>();
    extension_names_raw.push(DebugUtils::name().as_ptr()); // still wanna use the debug extensions

    let instance_create_info = vk::InstanceCreateInfo::builder()
        .push_next(&mut debugcreateinfo)
        .application_info(&app_info)
        .enabled_layer_names(&layers_names_raw)
        .enabled_extension_names(&extension_names_raw);
    unsafe { entry.create_instance(&instance_create_info, None) }
}

struct DebugMessenger {
    loader: DebugUtils,
    messenger: vk::DebugUtilsMessengerEXT,
}

impl DebugMessenger {
    fn init(entry: &ash::Entry, instance: &ash::Instance) -> Result<DebugMessenger, vk::Result> {
        // debug config
        let debugcreateinfo = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                    | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
            )
            .pfn_user_callback(Some(vulkan_debug_utils_callback));

        let loader = DebugUtils::new(entry, instance);
        let messenger = unsafe { loader.create_debug_utils_messenger(&debugcreateinfo, None)? };

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

struct SurfaceWrapper {
    surface: vk::SurfaceKHR,
    surface_loader: khr::Surface,
}

impl SurfaceWrapper {
    fn init(
        window: &Window,
        entry: &ash::Entry,
        instance: &ash::Instance,
    ) -> Result<SurfaceWrapper, vk::Result> {
        // load the surface
        // handles x11 or whatever OS specific drivers
        // this shit is terrible and nobody wants to do it, so lets use ash-window
        let surface = unsafe { ash_window::create_surface(entry, instance, window, None).unwrap() };
        let surface_loader = khr::Surface::new(entry, instance);
        Ok(SurfaceWrapper {
            surface,
            surface_loader,
        })
    }

    fn get_capabilities(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> Result<vk::SurfaceCapabilitiesKHR, vk::Result> {
        unsafe {
            self.surface_loader
                .get_physical_device_surface_capabilities(physical_device, self.surface)
        }
    }

    // fn get_present_modes(
    //     &self,
    //     physical_device: vk::PhysicalDevice,
    // ) -> Result<Vec<vk::PresentModeKHR>, vk::Result> {
    //     unsafe {
    //         self.surface_loader
    //             .get_physical_device_surface_present_modes(physical_device, self.surface)
    //     }
    // }

    fn get_formats(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> Result<Vec<vk::SurfaceFormatKHR>, vk::Result> {
        unsafe {
            self.surface_loader
                .get_physical_device_surface_formats(physical_device, self.surface)
        }
    }

    fn get_physical_device_surface_support(
        &self,
        physical_device: vk::PhysicalDevice,
        queuefamilyindex: usize,
    ) -> Result<bool, vk::Result> {
        unsafe {
            self.surface_loader.get_physical_device_surface_support(
                physical_device,
                queuefamilyindex as u32,
                self.surface,
            )
        }
    }
}

impl Drop for SurfaceWrapper {
    fn drop(&mut self) {
        unsafe {
            self.surface_loader.destroy_surface(self.surface, None);
        }
    }
}

// choose gpu
// https://hoj-senna.github.io/ashen-engine/text/004_Physical_device.html
fn init_physical_device_and_properties(
    instance: &ash::Instance,
) -> Result<(vk::PhysicalDevice, vk::PhysicalDeviceProperties), vk::Result> {
    let phys_devs = unsafe {
        instance
            .enumerate_physical_devices()
            .expect("Physical device error")
    };

    let mut chosen = None;
    for p in phys_devs {
        let properties = unsafe { instance.get_physical_device_properties(p) };
        // select ANY gpu, not only deticated ones
        // if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
        chosen = Some((p, properties));

        #[cfg(debug_assertions)]
        {
            let name = String::from(
                unsafe { CStr::from_ptr(properties.device_name.as_ptr()) }
                    .to_str()
                    .unwrap(),
            );
            println!("Gpu: {}", name);
        }

        // }
    }

    Ok(chosen.unwrap())
}

struct QueueFamilies {
    graphics_q_index: Option<u32>,
}

impl QueueFamilies {
    fn init(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surfaces: &SurfaceWrapper,
    ) -> Result<QueueFamilies, vk::Result> {
        let queuefamilyproperties =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
        // TODO: refactor
        let mut found_graphics_q_index = None;
        for (index, qfam) in queuefamilyproperties.iter().enumerate() {
            if qfam.queue_count > 0
                && qfam.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                && surfaces.get_physical_device_surface_support(physical_device, index)?
            {
                found_graphics_q_index = Some(index as u32);
                break;
            }
        }
        Ok(QueueFamilies {
            graphics_q_index: found_graphics_q_index,
        })
    }
}

struct Queues {
    graphics_queue: vk::Queue,
}

fn init_device_and_queues(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    queue_families: &QueueFamilies,
) -> Result<(ash::Device, Queues), vk::Result> {
    // select queues
    // https://hoj-senna.github.io/ashen-engine/text/005_Queues.html
    // in this case we only want one queue for now
    let queue_family_index = queue_families.graphics_q_index.unwrap();
    let device_extension_names_raw = [khr::Swapchain::name().as_ptr()];
    let features = vk::PhysicalDeviceFeatures {
        shader_clip_distance: 1,
        ..Default::default()
    };
    let priorities = [1.0];

    let queue_info = [vk::DeviceQueueCreateInfo::builder()
        .queue_family_index(queue_family_index)
        .queue_priorities(&priorities)
        .build()];

    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_info)
        .enabled_extension_names(&device_extension_names_raw)
        .enabled_features(&features);

    let logical_device: ash::Device = unsafe {
        instance
            .create_device(physical_device, &device_create_info, None)
            .unwrap()
    };

    let present_queue = unsafe { logical_device.get_device_queue(queue_family_index as u32, 0) };

    Ok((
        logical_device,
        Queues {
            graphics_queue: present_queue,
        },
    ))
}

#[allow(dead_code)]
struct SwapchainWrapper {
    swapchain_loader: ash::extensions::khr::Swapchain,
    swapchain: vk::SwapchainKHR,
    images: Vec<vk::Image>,
    imageviews: Vec<vk::ImageView>,
    framebuffers: Vec<vk::Framebuffer>,
    surface_format: vk::SurfaceFormatKHR,
    extent: vk::Extent2D,
    image_available: Vec<vk::Semaphore>,
    rendering_finished: Vec<vk::Semaphore>,
    may_begin_drawing: Vec<vk::Fence>,
    amount_of_images: u32,
    current_image: usize,
}

impl SwapchainWrapper {
    fn init(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        logical_device: &ash::Device,
        surfaces: &SurfaceWrapper,
        queue_families: &QueueFamilies,
    ) -> Result<SwapchainWrapper, vk::Result> {
        let surface_capabilities = surfaces.get_capabilities(physical_device)?;
        let extent = surface_capabilities.current_extent;
        let surface_format = *surfaces.get_formats(physical_device)?.first().unwrap();
        let queuefamilies = [queue_families.graphics_q_index.unwrap()];
        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surfaces.surface)
            .min_image_count(
                3.max(surface_capabilities.min_image_count)
                    .min(surface_capabilities.max_image_count),
            )
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&queuefamilies)
            .pre_transform(surface_capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(vk::PresentModeKHR::FIFO);
        let swapchain_loader = ash::extensions::khr::Swapchain::new(instance, logical_device);
        let swapchain = unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None)? };
        let swapchain_images = unsafe { swapchain_loader.get_swapchain_images(swapchain)? };
        let amount_of_images = swapchain_images.len() as u32;
        let mut swapchain_imageviews = Vec::with_capacity(swapchain_images.len());
        for image in &swapchain_images {
            let subresource_range = vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1);
            let imageview_create_info = vk::ImageViewCreateInfo::builder()
                .image(*image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(vk::Format::B8G8R8A8_UNORM)
                .subresource_range(*subresource_range);
            let imageview =
                unsafe { logical_device.create_image_view(&imageview_create_info, None) }?;
            swapchain_imageviews.push(imageview);
        }
        let mut image_available = vec![];
        let mut rendering_finished = vec![];
        let mut may_begin_drawing = vec![];
        let semaphoreinfo = vk::SemaphoreCreateInfo::builder();
        let fenceinfo = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
        for _ in 0..amount_of_images {
            let semaphore_available =
                unsafe { logical_device.create_semaphore(&semaphoreinfo, None) }?;
            let semaphore_finished =
                unsafe { logical_device.create_semaphore(&semaphoreinfo, None) }?;
            image_available.push(semaphore_available);
            rendering_finished.push(semaphore_finished);
            let fence = unsafe { logical_device.create_fence(&fenceinfo, None) }?;
            may_begin_drawing.push(fence);
        }
        Ok(SwapchainWrapper {
            swapchain_loader,
            swapchain,
            images: swapchain_images,
            imageviews: swapchain_imageviews,
            framebuffers: vec![],
            surface_format,
            extent,
            amount_of_images,
            current_image: 0,
            image_available,
            rendering_finished,
            may_begin_drawing,
        })
    }

    fn create_framebuffers(
        &mut self,
        logical_device: &ash::Device,
        renderpass: vk::RenderPass,
    ) -> Result<(), vk::Result> {
        for iv in &self.imageviews {
            let iview = [*iv];
            let framebuffer_info = vk::FramebufferCreateInfo::builder()
                .render_pass(renderpass)
                .attachments(&iview)
                .width(self.extent.width)
                .height(self.extent.height)
                .layers(1);
            let fb = unsafe { logical_device.create_framebuffer(&framebuffer_info, None) }?;
            self.framebuffers.push(fb);
        }
        Ok(())
    }

    unsafe fn cleanup(&mut self, logical_device: &ash::Device) {
        for fence in &self.may_begin_drawing {
            logical_device.destroy_fence(*fence, None);
        }
        for semaphore in &self.image_available {
            logical_device.destroy_semaphore(*semaphore, None);
        }
        for semaphore in &self.rendering_finished {
            logical_device.destroy_semaphore(*semaphore, None);
        }
        for fb in &self.framebuffers {
            logical_device.destroy_framebuffer(*fb, None);
        }
        for iv in &self.imageviews {
            logical_device.destroy_image_view(*iv, None);
        }
        self.swapchain_loader
            .destroy_swapchain(self.swapchain, None)
    }
}

impl Drop for SwapchainWrapper {
    fn drop(&mut self) {
        unsafe {
            // needs to be seperate function, because we depend on logical_device
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None)
        };
    }
}

fn init_renderpass(
    logical_device: &ash::Device,
    physical_device: vk::PhysicalDevice,
    surfaces: &SurfaceWrapper,
) -> Result<vk::RenderPass, vk::Result> {
    let attachments = [vk::AttachmentDescription::builder()
        .format(
            surfaces
                .get_formats(physical_device)?
                .first()
                .unwrap()
                .format,
        )
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
        .samples(vk::SampleCountFlags::TYPE_1)
        .build()];
    let color_attachment_references = [vk::AttachmentReference {
        attachment: 0,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    }];
    let subpasses = [vk::SubpassDescription::builder()
        .color_attachments(&color_attachment_references)
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .build()];
    let subpass_dependencies = [vk::SubpassDependency::builder()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_subpass(0)
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_access_mask(
            vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
        )
        .build()];
    let renderpass_info = vk::RenderPassCreateInfo::builder()
        .attachments(&attachments)
        .subpasses(&subpasses)
        .dependencies(&subpass_dependencies);
    let renderpass = unsafe { logical_device.create_render_pass(&renderpass_info, None)? };
    Ok(renderpass)
}

struct Pipeline {
    pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,
}

impl Pipeline {
    fn cleanup(&self, logical_device: &ash::Device) {
        unsafe {
            logical_device.destroy_pipeline(self.pipeline, None);
            logical_device.destroy_pipeline_layout(self.layout, None);
        }
    }

    fn init(
        logical_device: &ash::Device,
        swapchain: &SwapchainWrapper,
        renderpass: &vk::RenderPass,
    ) -> Result<Pipeline, vk::Result> {
        let vertexshader_createinfo = vk::ShaderModuleCreateInfo::builder().code(
            vk_shader_macros::include_glsl!("shaders/triangle.vert", kind: vert),
        );
        let vertexshader_module =
            unsafe { logical_device.create_shader_module(&vertexshader_createinfo, None)? };
        let fragmentshader_createinfo = vk::ShaderModuleCreateInfo::builder()
            .code(vk_shader_macros::include_glsl!("shaders/triangle.frag"));
        let fragmentshader_module =
            unsafe { logical_device.create_shader_module(&fragmentshader_createinfo, None)? };
        let mainfunctionname = std::ffi::CString::new("main").unwrap();
        let vertexshader_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vertexshader_module)
            .name(&mainfunctionname);
        let fragmentshader_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(fragmentshader_module)
            .name(&mainfunctionname);
        let shader_stages = vec![vertexshader_stage.build(), fragmentshader_stage.build()];

        let vertex_attrib_descs = [vk::VertexInputAttributeDescription {
            binding: 0,
            location: 0,
            offset: 0,
            format: vk::Format::R32G32B32A32_SFLOAT,
        }];
        let vertex_binding_descs = [vk::VertexInputBindingDescription {
            binding: 0,
            stride: 16,
            input_rate: vk::VertexInputRate::VERTEX,
        }];
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(&vertex_attrib_descs)
            .vertex_binding_descriptions(&vertex_binding_descs);

        let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::POINT_LIST);
        let viewports = [vk::Viewport {
            x: 0.,
            y: 0.,
            width: swapchain.extent.width as f32,
            height: swapchain.extent.height as f32,
            min_depth: 0.,
            max_depth: 1.,
        }];
        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: swapchain.extent,
        }];

        let viewport_info = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&viewports)
            .scissors(&scissors);
        let rasterizer_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .line_width(1.0)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .cull_mode(vk::CullModeFlags::NONE)
            .polygon_mode(vk::PolygonMode::FILL);
        let multisampler_info = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);
        let colourblend_attachments = [vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .alpha_blend_op(vk::BlendOp::ADD)
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
            )
            .build()];
        let colourblend_info =
            vk::PipelineColorBlendStateCreateInfo::builder().attachments(&colourblend_attachments);
        let pipelinelayout_info = vk::PipelineLayoutCreateInfo::builder();
        let pipelinelayout =
            unsafe { logical_device.create_pipeline_layout(&pipelinelayout_info, None) }?;
        let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly_info)
            .viewport_state(&viewport_info)
            .rasterization_state(&rasterizer_info)
            .multisample_state(&multisampler_info)
            .color_blend_state(&colourblend_info)
            .layout(pipelinelayout)
            .render_pass(*renderpass)
            .subpass(0);
        let graphicspipeline = unsafe {
            logical_device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[pipeline_info.build()],
                    None,
                )
                .expect("A problem with the pipeline creation")
        }[0];
        unsafe {
            logical_device.destroy_shader_module(fragmentshader_module, None);
            logical_device.destroy_shader_module(vertexshader_module, None);
        }
        Ok(Pipeline {
            pipeline: graphicspipeline,
            layout: pipelinelayout,
        })
    }
}

struct Pools {
    commandpool_graphics: vk::CommandPool,
}

impl Pools {
    fn init(
        logical_device: &ash::Device,
        queue_families: &QueueFamilies,
    ) -> Result<Pools, vk::Result> {
        let graphics_commandpool_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(queue_families.graphics_q_index.unwrap())
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        let commandpool_graphics =
            unsafe { logical_device.create_command_pool(&graphics_commandpool_info, None) }?;
        Ok(Pools {
            commandpool_graphics,
        })
    }
    fn cleanup(&self, logical_device: &ash::Device) {
        unsafe {
            logical_device.destroy_command_pool(self.commandpool_graphics, None);
        }
    }
}

fn create_commandbuffers(
    logical_device: &ash::Device,
    pools: &Pools,
    amount: usize,
) -> Result<Vec<vk::CommandBuffer>, vk::Result> {
    let commandbuf_allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(pools.commandpool_graphics)
        .command_buffer_count(amount as u32);
    unsafe { logical_device.allocate_command_buffers(&commandbuf_allocate_info) }
}

fn fill_commandbuffers(
    commandbuffers: &[vk::CommandBuffer],
    logical_device: &ash::Device,
    renderpass: &vk::RenderPass,
    swapchain: &SwapchainWrapper,
    pipeline: &Pipeline,
    vb: &vk::Buffer,
) -> Result<(), vk::Result> {
    for (i, &commandbuffer) in commandbuffers.iter().enumerate() {
        let commandbuffer_begininfo = vk::CommandBufferBeginInfo::builder();
        unsafe {
            logical_device.begin_command_buffer(commandbuffer, &commandbuffer_begininfo)?;
        }
        let clearvalues = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.08, 1.0],
            },
        }];
        let renderpass_begininfo = vk::RenderPassBeginInfo::builder()
            .render_pass(*renderpass)
            .framebuffer(swapchain.framebuffers[i])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: swapchain.extent,
            })
            .clear_values(&clearvalues);
        unsafe {
            logical_device.cmd_begin_render_pass(
                commandbuffer,
                &renderpass_begininfo,
                vk::SubpassContents::INLINE,
            );
            logical_device.cmd_bind_pipeline(
                commandbuffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.pipeline,
            );
            logical_device.cmd_bind_vertex_buffers(commandbuffer, 0, &[*vb], &[0]);
            logical_device.cmd_draw(commandbuffer, 1, 1, 0, 0);
            logical_device.cmd_end_render_pass(commandbuffer);
            logical_device.end_command_buffer(commandbuffer)?;
        }
    }
    Ok(())
}

#[allow(dead_code)]
struct Engine {
    window: winit::window::Window,
    entry: ash::Entry,
    instance: ash::Instance,
    debug: std::mem::ManuallyDrop<DebugMessenger>,
    surfaces: std::mem::ManuallyDrop<SurfaceWrapper>,
    physical_device: vk::PhysicalDevice,
    physical_device_properties: vk::PhysicalDeviceProperties,
    queue_families: QueueFamilies,
    queues: Queues,
    device: ash::Device,
    swapchain: SwapchainWrapper,
    renderpass: vk::RenderPass,
    pipeline: Pipeline,
    pools: Pools,
    commandbuffers: Vec<vk::CommandBuffer>,
    allocator: vk_mem::Allocator,
    buffer: vk::Buffer,
    allocation: vk_mem::Allocation,
    allocation_info: vk_mem::AllocationInfo,
}

impl Engine {
    fn init(window: winit::window::Window) -> Result<Engine, Box<dyn std::error::Error>> {
        let entry = ash::Entry::new()?;

        let instance = init_instance(&window, &entry)?;
        let debug = DebugMessenger::init(&entry, &instance)?;
        let surfaces = SurfaceWrapper::init(&window, &entry, &instance)?;

        let (physical_device, physical_device_properties) =
            init_physical_device_and_properties(&instance)?;

        let queue_families = QueueFamilies::init(&instance, physical_device, &surfaces)?;

        let (logical_device, queues) =
            init_device_and_queues(&instance, physical_device, &queue_families)?;

        let mut swapchain = SwapchainWrapper::init(
            &instance,
            physical_device,
            &logical_device,
            &surfaces,
            &queue_families,
        )?;
        let renderpass = init_renderpass(&logical_device, physical_device, &surfaces)?;
        swapchain.create_framebuffers(&logical_device, renderpass)?;
        let pipeline = Pipeline::init(&logical_device, &swapchain, &renderpass)?;
        let pools = Pools::init(&logical_device, &queue_families)?;

        let allocator_create_info = vk_mem::AllocatorCreateInfo {
            physical_device,
            device: logical_device.clone(),
            instance: instance.clone(),
            ..Default::default()
        };
        let allocator = vk_mem::Allocator::new(&allocator_create_info)?;
        let allocation_create_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::CpuToGpu,
            ..Default::default()
        };

        let data = [0.0f32, 0.0f32, 0.0f32, 0.0f32];

        let (buffer, allocation, allocation_info) = allocator.create_buffer(
            &vk::BufferCreateInfo::builder()
                .size((data.len() * std::mem::size_of::<f32>()) as u64)
                .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
                .build(),
            &allocation_create_info,
        )?;
        let data_ptr = allocator.map_memory(&allocation)? as *mut f32;
        // TODO: make struct for vertex data
        unsafe { data_ptr.copy_from_nonoverlapping(data.as_ptr(), data.len()) };
        allocator.unmap_memory(&allocation);

        let commandbuffers =
            create_commandbuffers(&logical_device, &pools, swapchain.framebuffers.len())?;

        fill_commandbuffers(
            &commandbuffers,
            &logical_device,
            &renderpass,
            &swapchain,
            &pipeline,
            &buffer,
        )?;

        Ok(Engine {
            window,
            entry,
            instance,
            debug: std::mem::ManuallyDrop::new(debug),
            surfaces: std::mem::ManuallyDrop::new(surfaces),
            physical_device,
            physical_device_properties,
            queue_families,
            queues,
            device: logical_device,
            swapchain,
            renderpass,
            pipeline,
            pools,
            commandbuffers,
            allocator,
            buffer,
            allocation,
            allocation_info,
        })
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        unsafe {
            self.device
                .device_wait_idle()
                .expect("something wrong while waiting");
            // if we fail to destroy the buffer continue to destory as many things
            // as possible
            let _ = self.allocator.destroy_buffer(self.buffer, &self.allocation);
            self.allocator.destroy();
            self.pools.cleanup(&self.device);
            self.pipeline.cleanup(&self.device);
            self.device.destroy_render_pass(self.renderpass, None);
            self.swapchain.cleanup(&self.device);
            self.device.destroy_device(None);
            std::mem::ManuallyDrop::drop(&mut self.surfaces);
            std::mem::ManuallyDrop::drop(&mut self.debug);
            self.instance.destroy_instance(None)
        };
    }
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

    match severity.as_str() {
        "error" => log::error!("[{}] {:?}", ty, message),
        "warn" => log::warn!("[{}] {:?}", ty, message),
        "info" => log::info!("[{}] {:?}", ty, message),
        "verbose" => log::trace!("[{}] {:?}", ty, message),
        _ => log::error!("Unknown severity ({}; message: {:?})", severity, message),
    };

    vk::FALSE
}
