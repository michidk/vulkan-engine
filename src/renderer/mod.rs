pub mod buffer;
pub mod camera;
mod debug;
mod descriptor_manager;
mod instance_device_queues;
pub mod light;
pub mod model;
mod pools_and_commandbuffers;
mod renderpass_and_pipeline;
mod shader;
mod surface;
mod swapchain;
pub mod texture;

use ash::{
    version::{DeviceV1_0, InstanceV1_0},
    vk,
};
use crystal::prelude::*;

use self::{
    buffer::BufferWrapper,
    debug::DebugMessenger,
    descriptor_manager::DescriptorManager,
    instance_device_queues::{QueueFamilies, Queues},
    model::{DefaultModel, TextureQuadModel},
    pools_and_commandbuffers::PoolsWrapper,
    renderpass_and_pipeline::PipelineWrapper,
    surface::SurfaceWrapper,
    swapchain::SwapchainWrapper,
    texture::TextureStorage,
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

pub fn screenshot(renderer: &Renderer) -> Result<(), Box<dyn std::error::Error>> {
    let commandbuf_allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(renderer.pools.commandpool_graphics)
        .command_buffer_count(1);
    let copy_buffer = unsafe {
        renderer
            .device
            .allocate_command_buffers(&commandbuf_allocate_info)
    }?[0];

    let cmd_begin_info =
        vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    unsafe {
        renderer
            .device
            .begin_command_buffer(copy_buffer, &cmd_begin_info)
    }?;

    let image_create_info = vk::ImageCreateInfo::builder()
        .format(vk::Format::R8G8B8A8_UNORM)
        .image_type(vk::ImageType::TYPE_2D)
        .extent(vk::Extent3D {
            width: renderer.swapchain.extent.width,
            height: renderer.swapchain.extent.height,
            depth: 1,
        })
        .array_layers(1)
        .mip_levels(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .tiling(vk::ImageTiling::LINEAR)
        .usage(vk::ImageUsageFlags::TRANSFER_DST)
        .initial_layout(vk::ImageLayout::UNDEFINED);

    let allocation_create_info = vk_mem::AllocationCreateInfo {
        usage: vk_mem::MemoryUsage::GpuToCpu,
        ..Default::default()
    };

    let (destination_image, destination_allocation, _allocation_info) = renderer
        .allocator
        .create_image(&image_create_info, &allocation_create_info)?;

    let destination_barrier = vk::ImageMemoryBarrier::builder()
        .image(destination_image)
        .src_access_mask(vk::AccessFlags::empty())
        .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
        .old_layout(vk::ImageLayout::UNDEFINED)
        .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        })
        .build();

    unsafe {
        renderer.device.cmd_pipeline_barrier(
            copy_buffer,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[destination_barrier],
        )
    };

    let source_image = renderer.swapchain.images[renderer.swapchain.current_image];
    let source_barrier = vk::ImageMemoryBarrier::builder()
        .image(source_image)
        .src_access_mask(vk::AccessFlags::MEMORY_READ)
        .dst_access_mask(vk::AccessFlags::TRANSFER_READ)
        .old_layout(vk::ImageLayout::PRESENT_SRC_KHR)
        .new_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        })
        .build();

    unsafe {
        renderer.device.cmd_pipeline_barrier(
            copy_buffer,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[source_barrier],
        )
    };

    let zero_offset = vk::Offset3D::default();
    let copy_area = vk::ImageCopy::builder()
        .src_subresource(vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            mip_level: 0,
            base_array_layer: 0,
            layer_count: 1,
        })
        .src_offset(zero_offset)
        .dst_subresource(vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            mip_level: 0,
            base_array_layer: 0,
            layer_count: 1,
        })
        .dst_offset(zero_offset)
        .extent(vk::Extent3D {
            width: renderer.swapchain.extent.width,
            height: renderer.swapchain.extent.height,
            depth: 1,
        })
        .build();

    unsafe {
        renderer.device.cmd_copy_image(
            copy_buffer,
            source_image,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            destination_image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &[copy_area],
        )
    };

    // make destination_image readable
    let destination_barrier = vk::ImageMemoryBarrier::builder()
        .image(destination_image)
        .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
        .dst_access_mask(vk::AccessFlags::MEMORY_READ)
        .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
        .new_layout(vk::ImageLayout::GENERAL)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        })
        .build();

    unsafe {
        renderer.device.cmd_pipeline_barrier(
            copy_buffer,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[destination_barrier],
        )
    };

    // reset source_image
    let source_barrier = vk::ImageMemoryBarrier::builder()
        .image(source_image)
        .src_access_mask(vk::AccessFlags::TRANSFER_READ)
        .dst_access_mask(vk::AccessFlags::MEMORY_READ)
        .old_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
        .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        })
        .build();
    unsafe {
        renderer.device.cmd_pipeline_barrier(
            copy_buffer,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[source_barrier],
        )
    };

    unsafe { renderer.device.end_command_buffer(copy_buffer) }?;

    let submit_infos = [vk::SubmitInfo::builder()
        .command_buffers(&[copy_buffer])
        .build()];
    let fence = unsafe {
        renderer
            .device
            .create_fence(&vk::FenceCreateInfo::default(), None)
    }?;
    unsafe {
        renderer
            .device
            .queue_submit(renderer.queues.graphics_queue, &submit_infos, fence)
    }?;

    // wait for fence
    unsafe { renderer.device.wait_for_fences(&[fence], true, u64::MAX) }?;
    unsafe { renderer.device.destroy_fence(fence, None) };
    unsafe {
        renderer
            .device
            .free_command_buffers(renderer.pools.commandpool_graphics, &[copy_buffer])
    };

    let source_ptr = renderer.allocator.map_memory(&destination_allocation)? as *mut u8;
    // get image size
    let subresource_layout = unsafe {
        renderer.device.get_image_subresource_layout(
            destination_image,
            vk::ImageSubresource {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                array_layer: 0,
            },
        )
    };

    let mut data = Vec::<u8>::with_capacity(subresource_layout.size as usize);
    unsafe {
        std::ptr::copy(
            source_ptr,
            data.as_mut_ptr(),
            subresource_layout.size as usize,
        );
        data.set_len(subresource_layout.size as usize);
    }

    // cleanup destination_image
    renderer.allocator.unmap_memory(&destination_allocation);
    renderer
        .allocator
        .destroy_image(destination_image, &destination_allocation);

    let screen: image::ImageBuffer<image::Bgra<u8>, _> = image::ImageBuffer::from_raw(
        renderer.swapchain.extent.width,
        renderer.swapchain.extent.height,
        data,
    )
    .expect("ImageBuffer creation");

    let screen_image = image::DynamicImage::ImageBgra8(screen).to_rgba8();
    screen_image.save(format!(
        "screenshot_{}.jpg",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_secs()
    ))?;

    Ok(())
}

#[allow(dead_code)]
pub struct Renderer {
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
    pub light_buffer: BufferWrapper,
    pub texture_storage: TextureStorage,
    descriptor_manager: DescriptorManager<8>,
    layout_camera: vk::DescriptorSetLayout,
    layout_lights: vk::DescriptorSetLayout,
}

impl Renderer {
    pub fn init(window: winit::window::Window) -> Result<Renderer, Box<dyn std::error::Error>> {
        let entry = ash::Entry::new()?;

        let instance = instance_device_queues::init_instance(&window, &entry)?;
        let debug = DebugMessenger::init(&entry, &instance)?;
        let surface = SurfaceWrapper::init(&window, &entry, &instance);

        let (physical_device, physical_device_properties, _physical_device_features) =
            instance_device_queues::init_physical_device_and_properties(&instance)?;

        let queue_families = QueueFamilies::init(&instance, physical_device, &surface)?;

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
            &surface,
            &queue_families,
            &allocator,
        )?;

        let format = surface.choose_format(physical_device)?.format;
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

        let layout_camera = pipeline.descriptor_set_layouts[0];

        let mut light_buffer = BufferWrapper::new(
            &allocator,
            8,
            vk::BufferUsageFlags::STORAGE_BUFFER,
            vk_mem::MemoryUsage::CpuToGpu,
        )?;
        light_buffer.fill(&allocator, &[0.0, 0.0])?;

        // let descriptor_sets_light = vec![];
        let layout_lights = pipeline.descriptor_set_layouts[1];

        // let desc_layouts_texture =
        //     vec![pipeline.descriptor_set_layouts[1]; swapchain.amount_of_images as usize];
        // let descriptor_set_allocate_info_texture = vk::DescriptorSetAllocateInfo::builder()
        //     .descriptor_pool(descriptor_pool)
        //     .set_layouts(&desc_layouts_texture);
        // let descriptor_sets_texture = unsafe {
        //     logical_device.allocate_descriptor_sets(&descriptor_set_allocate_info_texture)
        // }?;

        let descriptor_manager = DescriptorManager::new(logical_device.clone())?;

        Ok(Renderer {
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
            light_buffer,
            texture_storage: TextureStorage::new(),
            descriptor_manager,
            layout_camera,
            layout_lights,
        })
    }

    pub fn update_commandbuffer(&mut self, index: usize) -> Result<(), vk::Result> {
        self.descriptor_manager.next_frame();

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

            let cam_set_data = vec![descriptor_manager::DescriptorData::UniformBuffer {
                buffer: self.uniform_buffer.buffer,
                offset: 0,
                size: self.uniform_buffer.get_size(),
            }];
            let cam_set = self
                .descriptor_manager
                .get_descriptor_set(self.layout_camera, &cam_set_data)?;

            let light_set_data = vec![descriptor_manager::DescriptorData::StorageBuffer {
                buffer: self.light_buffer.buffer,
                offset: 0,
                size: self.light_buffer.get_size(),
            }];
            let light_set = self
                .descriptor_manager
                .get_descriptor_set(self.layout_lights, &light_set_data)?;

            self.device.cmd_bind_descriptor_sets(
                commandbuffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.layout,
                0,
                &[
                    cam_set, light_set,
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

    pub fn new_texture_from_file<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        self.texture_storage.new_texture_from_file(
            path,
            &self.device,
            &self.allocator,
            &self.pools.commandpool_graphics,
            &self.queues.graphics_queue,
        )
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.device
                .device_wait_idle()
                .expect("something wrong while waiting");

            self.descriptor_manager.destroy();

            self.texture_storage.cleanup(&self.device, &self.allocator);

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
