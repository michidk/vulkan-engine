use std::ffi::CString;

use ash::{
    extensions::ext,
    version::{DeviceV1_0, EntryV1_0, InstanceV1_0},
    vk,
};
use math::prelude::Mat4;

use self::queue::QueueFamilies;
use super::renderpass;
use crate::{
    engine::Info,
    scene::{
        light::LightManager,
        model::model::{DefaultModel, TextureQuadModel},
    },
};

use super::{
    buffer::BufferWrapper,
    debug::{self, DebugMessenger},
    device,
    pipeline::PipelineWrapper,
    queue::{self, PoolsWrapper, Queues},
    surface::SurfaceWrapper,
    swapchain::SwapchainWrapper,
};

pub struct VulkanManager {
    pub window: winit::window::Window,
    entry: ash::Entry,
    instance: ash::Instance,
    debug: std::mem::ManuallyDrop<DebugMessenger>,
    surface: std::mem::ManuallyDrop<SurfaceWrapper>,
    physical_device: vk::PhysicalDevice,
    physical_device_properties: vk::PhysicalDeviceProperties,
    queue_families: QueueFamilies,
    pub queues: Queues,
    pub device: ash::Device,
    pub swapchain: SwapchainWrapper,
    renderpass: vk::RenderPass,
    pipeline: PipelineWrapper,
    pub pools: PoolsWrapper,
    pub commandbuffers: Vec<vk::CommandBuffer>,
    pub allocator: vk_mem::Allocator,
    pub models: Vec<DefaultModel>,
    pub texture_quads: Vec<TextureQuadModel>,
    pub uniform_buffer: BufferWrapper,
    descriptor_pool: vk::DescriptorPool,
    descriptor_sets_camera: Vec<vk::DescriptorSet>,
    pub descriptor_sets_light: Vec<vk::DescriptorSet>,
    pub light_buffer: BufferWrapper,
}

impl VulkanManager {
    pub fn new(
        engine_info: Info,
        window: winit::window::Window,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let entry = ash::Entry::new()?;

        let instance = VulkanManager::init_instance(engine_info, &entry, &window)?;
        let debug = DebugMessenger::init(&entry, &instance)?;
        let surface = SurfaceWrapper::init(&window, &entry, &instance);

        let (physical_device, physical_device_properties, _physical_device_features) =
            device::select_physical_device(&instance)?;

        let queue_families = QueueFamilies::init(&instance, physical_device, &surface)?;

        let (logical_device, queues) =
            queue::init_device_and_queues(&instance, physical_device, &queue_families)?;

        let allocator_create_info = vk_mem::AllocatorCreateInfo {
            physical_device,
            device: logical_device.clone(),
            instance: instance.clone(),
            ..Default::default()
        };
        let allocator = vk_mem::Allocator::new(&allocator_create_info)?;

        let mut swapchain = SwapchainWrapper::init(
            &instance,
            physical_device,
            &logical_device,
            &surface,
            &queue_families,
            &allocator,
        )?;

        let format = surface.choose_format(physical_device)?.format;
        let renderpass = renderpass::init_renderpass(&logical_device, format)?;
        swapchain.create_framebuffers(&logical_device, renderpass)?;
        let pipeline = PipelineWrapper::init(&logical_device, &swapchain, &renderpass)?;
        let pools = PoolsWrapper::init(&logical_device, &queue_families)?;

        let commandbuffers =
            queue::create_commandbuffers(&logical_device, &pools, swapchain.framebuffers.len())?;

        let mut uniform_buffer = BufferWrapper::new(
            &allocator,
            128,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk_mem::MemoryUsage::CpuToGpu,
        )?;
        let camera_transform: [[[f32; 4]; 4]; 2] =
            [Mat4::identity().into(), Mat4::identity().into()];
        uniform_buffer.fill(&allocator, &camera_transform)?;

        let pool_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: swapchain.amount_of_images,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: swapchain.amount_of_images,
            },
        ];
        let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(2 * swapchain.amount_of_images)
            .pool_sizes(&pool_sizes);
        let descriptor_pool =
            unsafe { logical_device.create_descriptor_pool(&descriptor_pool_info, None) }?;

        let desc_layouts_camera =
            vec![pipeline.descriptor_set_layouts[0]; swapchain.amount_of_images as usize];
        let descriptor_set_allocate_info_camera = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&desc_layouts_camera);
        let descriptor_sets_camera = unsafe {
            logical_device.allocate_descriptor_sets(&descriptor_set_allocate_info_camera)
        }?;
        for descset in &descriptor_sets_camera {
            let buffer_infos = [vk::DescriptorBufferInfo {
                buffer: uniform_buffer.buffer,
                offset: 0,
                range: 128,
            }];
            let desc_set_write = [vk::WriteDescriptorSet::builder()
                .dst_set(*descset)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&buffer_infos)
                .build()];
            unsafe { logical_device.update_descriptor_sets(&desc_set_write, &[]) };
        }

        let mut light_buffer = BufferWrapper::new(
            &allocator,
            8,
            vk::BufferUsageFlags::STORAGE_BUFFER,
            vk_mem::MemoryUsage::CpuToGpu,
        )?;
        light_buffer.fill(&allocator, &[0.0, 0.0])?;

        // let descriptor_sets_light = vec![];
        let desc_layouts_light =
            vec![pipeline.descriptor_set_layouts[1]; swapchain.amount_of_images as usize];
        let descriptor_set_allocate_info_light = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&desc_layouts_light);
        let descriptor_sets_light = unsafe {
            logical_device.allocate_descriptor_sets(&descriptor_set_allocate_info_light)
        }?;
        for descset in &descriptor_sets_light {
            let buffer_infos = [vk::DescriptorBufferInfo {
                buffer: light_buffer.buffer,
                offset: 0,
                range: 8,
            }];
            let desc_set_write = [vk::WriteDescriptorSet::builder()
                .dst_set(*descset)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .buffer_info(&buffer_infos)
                .build()];
            unsafe { logical_device.update_descriptor_sets(&desc_set_write, &[]) };
        }

        Ok(Self {
            window,
            entry,
            instance,
            debug: std::mem::ManuallyDrop::new(debug),
            surface: std::mem::ManuallyDrop::new(surface),
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
            models: Vec::new(),
            texture_quads: Vec::new(),
            uniform_buffer,
            descriptor_pool,
            descriptor_sets_camera,
            descriptor_sets_light,
            light_buffer,
        })
    }

    fn init_instance(
        engine_info: Info,
        entry: &ash::Entry,
        window: &winit::window::Window,
    ) -> Result<ash::Instance, ash::InstanceError> {
        let app_name = CString::new(engine_info.app_name).unwrap();

        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .application_version(vk::make_version(0, 0, 1))
            .engine_name(&app_name)
            .engine_version(vk::make_version(0, 0, 1))
            .api_version(vk::make_version(1, 2, 0));

        let surface_extensions = ash_window::enumerate_required_extensions(window).unwrap();
        let mut extension_names_raw = surface_extensions
            .iter()
            .map(|ext| ext.as_ptr())
            .collect::<Vec<_>>();
        extension_names_raw.push(ext::DebugUtils::name().as_ptr()); // still wanna use the debug extensions

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

    pub fn update_commandbuffer(&mut self, index: usize) -> Result<(), vk::Result> {
        let commandbuffer = self.commandbuffers[index];
        let commandbuffer_begininfo = vk::CommandBufferBeginInfo::builder();
        unsafe {
            self.device
                .begin_command_buffer(commandbuffer, &commandbuffer_begininfo)?;
        }

        let clearvalues = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.003_861_873, 1.0],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        ];
        let renderpass_begininfo = vk::RenderPassBeginInfo::builder()
            .render_pass(self.renderpass)
            .framebuffer(self.swapchain.framebuffers[index])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.extent,
            })
            .clear_values(&clearvalues);
        unsafe {
            self.device.cmd_begin_render_pass(
                commandbuffer,
                &renderpass_begininfo,
                vk::SubpassContents::INLINE,
            );
            self.device.cmd_bind_pipeline(
                commandbuffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.pipeline,
            );
            self.device.cmd_bind_descriptor_sets(
                commandbuffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.layout,
                0,
                &[
                    self.descriptor_sets_camera[index],
                    self.descriptor_sets_light[index],
                    // self.descriptor_sets_texture[index],
                ],
                &[],
            );
            for m in &self.models {
                m.draw(&self.device, commandbuffer);
            }
            for m in &self.texture_quads {
                m.draw(&self.device, commandbuffer);
            }
            self.device.cmd_end_render_pass(commandbuffer);
            self.device.end_command_buffer(commandbuffer)?;
        }
        Ok(())
    }

    pub fn recreate_swapchain(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            self.device
                .device_wait_idle()
                .expect("something went wrong while waiting");
        }
        unsafe {
            self.swapchain.cleanup(&self.device, &self.allocator);
        }
        self.swapchain = SwapchainWrapper::init(
            &self.instance,
            self.physical_device,
            &self.device,
            &self.surface,
            &self.queue_families,
            &self.allocator,
        )?;
        self.swapchain
            .create_framebuffers(&self.device, self.renderpass)?;
        self.pipeline.cleanup(&self.device);
        self.pipeline = PipelineWrapper::init(&self.device, &self.swapchain, &self.renderpass)?;
        Ok(())
    }

    pub fn wait_for_fence(&self) {
        unsafe {
            &self
                .device
                .wait_for_fences(
                    &[self.swapchain.may_begin_drawing[self.swapchain.current_image]],
                    true,
                    std::u64::MAX,
                )
                .expect("fence-waiting");
            &self
                .device
                .reset_fences(&[self.swapchain.may_begin_drawing[self.swapchain.current_image]])
                .expect("resetting fences");
        }
    }

    pub fn render(&self, image_index: u32) -> [vk::Semaphore; 1] {
        let semaphores_available = [self.swapchain.image_available[self.swapchain.current_image]];
        let waiting_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let semaphores_finished = [self.swapchain.rendering_finished[self.swapchain.current_image]];
        let commandbuffers = [self.commandbuffers[image_index as usize]];
        let submit_info = [vk::SubmitInfo::builder()
            .wait_semaphores(&semaphores_available)
            .wait_dst_stage_mask(&waiting_stages)
            .command_buffers(&commandbuffers)
            .signal_semaphores(&semaphores_finished)
            .build()];
        unsafe {
            &self
                .device
                .queue_submit(
                    self.queues.graphics_queue,
                    &submit_info,
                    self.swapchain.may_begin_drawing[self.swapchain.current_image],
                )
                .expect("queue submission");
        };

        semaphores_finished
    }

    pub fn present(&mut self, image_index: u32, semaphores_finished: &[vk::Semaphore]) {
        let swapchains = [self.swapchain.swapchain];
        let indices = [image_index];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(semaphores_finished)
            .swapchains(&swapchains)
            .image_indices(&indices);
        unsafe {
            match &self
                .swapchain
                .swapchain_loader
                .queue_present(self.queues.graphics_queue, &present_info)
            {
                Ok(..) => {}
                Err(ash::vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    self.recreate_swapchain().expect("swapchain recreation");
                    // camera.set_aspect(
                    //     vk.swapchain.extent.width as f32 / vk.swapchain.extent.height as f32,
                    // );
                    // camera.update_buffer(&vk.allocator, &mut vk.uniform_buffer);
                }
                _ => {
                    panic!("unhandled queue presentation error");
                }
            }
        };

        self.swapchain.current_image =
            (self.swapchain.current_image + 1) % self.swapchain.amount_of_images as usize;
    }
}

impl Drop for VulkanManager {
    fn drop(&mut self) {
        unsafe {
            self.device
                .device_wait_idle()
                .expect("something wrong while waiting");

            self.device
                .destroy_descriptor_pool(self.descriptor_pool, None);
            self.uniform_buffer.cleanup(&self.allocator);
            self.light_buffer.cleanup(&self.allocator);

            // if we fail to destroy the buffer continue to destory as many things
            // as possible
            for m in &mut self.models {
                m.cleanup(&self.allocator);
            }
            for m in &mut self.texture_quads {
                m.cleanup(&self.allocator);
            }

            self.pools.cleanup(&self.device);
            self.pipeline.cleanup(&self.device);
            self.device.destroy_render_pass(self.renderpass, None);
            // --segfault
            self.swapchain.cleanup(&self.device, &self.allocator);
            self.allocator.destroy();
            self.device.destroy_device(None);
            // --segfault
            std::mem::ManuallyDrop::drop(&mut self.surface);
            std::mem::ManuallyDrop::drop(&mut self.debug);
            self.instance.destroy_instance(None)
        };
    }
}
