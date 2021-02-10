mod buffer;
pub mod camera;
mod debug;
mod instance_device_queues;
pub mod model;
mod pools_and_commandbuffers;
mod renderpass_and_pipeline;
mod surface;
mod swapchain;

use ash::{
    version::{DeviceV1_0, InstanceV1_0},
    vk,
};
use math::prelude::*;

use self::{
    buffer::BufferWrapper,
    debug::DebugMessenger,
    instance_device_queues::{QueueFamilies, Queues},
    model::DefaultModel,
    pools_and_commandbuffers::PoolsWrapper,
    renderpass_and_pipeline::PipelineWrapper,
    surface::SurfaceWrapper,
    swapchain::SwapchainWrapper,
};

#[derive(thiserror::Error, Debug)]
pub enum RendererError {
    #[error("Unknown error")]
    Unknown,
    #[error("Vulkan error: {0}")]
    VkError(#[from] vk::Result),
    #[error("VulkanMemory error: {0}")]
    VkMemError(#[from] vk_mem::error::Error),
    #[error("No suitable gpu found")]
    NoSuitableGpu,
    #[error("No suitable queue family found")]
    NoSuitableQueueFamily,
    #[error("Invalid handle")]
    InvalidHandle,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AppInfo {
    width: u32,
    height: u32,
    title: &'static str,
}

impl AppInfo {
    pub fn into_window<T: 'static>(
        self,
        window_target: &winit::event_loop::EventLoopWindowTarget<T>,
    ) -> Result<winit::window::Window, winit::error::OsError> {
        winit::window::WindowBuilder::new()
            .with_title(self.title)
            .with_inner_size(winit::dpi::LogicalSize::new(
                f64::from(self.width),
                f64::from(self.height),
            ))
            .build(window_target)
    }
}

pub const DEFAULT_WINDOW_INFO: AppInfo = AppInfo {
    width: 1920,
    height: 1080,
    title: "VulkanTriangle",
};

#[allow(dead_code)]
pub struct Renderer {
    pub window: winit::window::Window,
    entry: ash::Entry,
    instance: ash::Instance,
    debug: std::mem::ManuallyDrop<DebugMessenger>,
    surfaces: std::mem::ManuallyDrop<SurfaceWrapper>,
    physical_device: vk::PhysicalDevice,
    physical_device_properties: vk::PhysicalDeviceProperties,
    queue_families: QueueFamilies,
    pub queues: Queues,
    pub device: ash::Device,
    pub swapchain: SwapchainWrapper,
    renderpass: vk::RenderPass,
    pipeline: PipelineWrapper,
    pools: PoolsWrapper,
    pub commandbuffers: Vec<vk::CommandBuffer>,
    pub allocator: vk_mem::Allocator,
    pub models: Vec<DefaultModel>,
    pub uniform_buffer: BufferWrapper,
    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: Vec<vk::DescriptorSet>,
}

impl Renderer {
    pub fn init(window: winit::window::Window) -> Result<Renderer, Box<dyn std::error::Error>> {
        let entry = ash::Entry::new()?;

        let instance = instance_device_queues::init_instance(&window, &entry)?;
        let debug = DebugMessenger::init(&entry, &instance)?;
        let surfaces = SurfaceWrapper::init(&window, &entry, &instance);

        let (physical_device, physical_device_properties, _physical_device_features) =
            instance_device_queues::init_physical_device_and_properties(&instance)?;

        let queue_families = QueueFamilies::init(&instance, physical_device, &surfaces)?;

        let (logical_device, queues) = instance_device_queues::init_device_and_queues(
            &instance,
            physical_device,
            &queue_families,
        )?;

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
            &surfaces,
            &queue_families,
            &allocator,
        )?;
        let format = surfaces
            .get_formats(physical_device)?
            .first()
            .unwrap()
            .format;
        let renderpass = renderpass_and_pipeline::init_renderpass(&logical_device, format)?;
        swapchain.create_framebuffers(&logical_device, renderpass)?;
        let pipeline = PipelineWrapper::init(&logical_device, &swapchain, &renderpass)?;
        let pools = PoolsWrapper::init(&logical_device, &queue_families)?;

        let commandbuffers = pools_and_commandbuffers::create_commandbuffers(
            &logical_device,
            &pools,
            swapchain.framebuffers.len(),
        )?;

        let mut uniform_buffer = BufferWrapper::new(
            &allocator,
            128,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk_mem::MemoryUsage::CpuToGpu,
        )?;
        let camera_transform: [[[f32; 4]; 4]; 2] =
            [Mat4::identity().into(), Mat4::identity().into()];
        uniform_buffer.fill(&allocator, &camera_transform)?;

        let pool_size = [vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: swapchain.amount_of_images,
        }];
        let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(swapchain.amount_of_images)
            .pool_sizes(&pool_size);
        let descriptor_pool =
            unsafe { logical_device.create_descriptor_pool(&descriptor_pool_info, None) }?;
        let desc_layouts =
            vec![pipeline.descriptor_set_layouts[0]; swapchain.amount_of_images as usize];
        let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&desc_layouts);
        let descriptor_sets =
            unsafe { logical_device.allocate_descriptor_sets(&descriptor_set_allocate_info) }?;
        for descset in &descriptor_sets {
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

        Ok(Renderer {
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
            models: Vec::new(),
            uniform_buffer,
            descriptor_pool,
            descriptor_sets,
        })
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
                    float32: [0.0, 0.0, 0.08, 1.0],
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
                &[self.descriptor_sets[index]],
                &[],
            );
            for m in &self.models {
                m.draw(&self.device, commandbuffer);
            }
            self.device.cmd_end_render_pass(commandbuffer);
            self.device.end_command_buffer(commandbuffer)?;
        }
        Ok(())
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.device
                .device_wait_idle()
                .expect("something wrong while waiting");

            self.device
                .destroy_descriptor_pool(self.descriptor_pool, None);
            self.uniform_buffer.cleanup(&self.allocator);

            // if we fail to destroy the buffer continue to destory as many things
            // as possible
            for m in &mut self.models {
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
            std::mem::ManuallyDrop::drop(&mut self.surfaces);
            std::mem::ManuallyDrop::drop(&mut self.debug);
            self.instance.destroy_instance(None)
        };
    }
}
