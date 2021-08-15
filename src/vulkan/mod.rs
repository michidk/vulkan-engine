pub(crate) mod buffer;
mod debug;
pub mod descriptor_manager;
mod device;
pub mod error;
pub mod lighting_pipeline;
pub mod pipeline;
pub mod pp_effect;
mod queue;
mod renderpass;
mod surface;
mod swapchain;
pub mod texture;
pub mod uploader;
pub mod allocator;

use std::{ffi::CString, mem::size_of, ptr::null, rc::Rc, slice};

use ash::{extensions::{ext, khr}, vk::{self, Handle}};
use gpu_allocator::SubAllocation;

use crate::{
    core::camera::{self, CamData},
    core::engine::EngineInfo,
    scene::{
        light::LightManager,
        material::Material,
        model::{mesh::Mesh, Model},
        Scene,
    },
};

use self::{allocator::Allocator, buffer::{PerFrameUniformBuffer, VulkanBuffer}, debug::DebugMessenger, descriptor_manager::{DescriptorData, DescriptorManager}, error::GraphicsResult, lighting_pipeline::LightingPipeline, pp_effect::PPEffect, queue::{PoolsWrapper, QueueFamilies, Queues}, surface::SurfaceWrapper, swapchain::SwapchainWrapper, uploader::Uploader};

pub struct VulkanManager {
    #[allow(dead_code)]
    entry: ash::Entry,
    instance: ash::Instance,
    pub allocator: std::mem::ManuallyDrop<Rc<Allocator>>,
    pub device: Rc<ash::Device>,

    debug: std::mem::ManuallyDrop<DebugMessenger>,
    surface: std::mem::ManuallyDrop<SurfaceWrapper>,
    physical_device: vk::PhysicalDevice,
    #[allow(dead_code)]
    physical_device_properties: vk::PhysicalDeviceProperties,
    queue_families: QueueFamilies,
    pub queues: Queues,
    pub swapchain: SwapchainWrapper,
    pub renderpass: vk::RenderPass,
    pub pools: PoolsWrapper,
    pub commandbuffers: Vec<vk::CommandBuffer>,
    pub uniform_buffer: PerFrameUniformBuffer<camera::CamData>,
    pub desc_layout_frame_data: vk::DescriptorSetLayout,
    pipeline_layout_gpass: vk::PipelineLayout,
    pub pipeline_layout_resolve_pass: vk::PipelineLayout,
    pub descriptor_manager: DescriptorManager<8>,
    max_frames_in_flight: u8,
    pub current_frame_index: u8,
    image_acquire_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    frame_resource_fences: Vec<vk::Fence>,
    lighting_pipelines: Vec<Rc<LightingPipeline>>,
    sampler_linear: vk::Sampler,
    desc_layout_pp: vk::DescriptorSetLayout,
    pub pipe_layout_pp: vk::PipelineLayout,
    pub renderpass_pp: vk::RenderPass,
    pp_effects: Vec<Rc<PPEffect>>,
    pub uploader: std::mem::ManuallyDrop<Uploader>,
    pub enable_wireframe: bool,

    pub rtx_data: Option<RtxData>,
}

pub struct RtxData {
    pub acc_ext: Rc<khr::AccelerationStructure>,
    pub rtx_ext: Rc<khr::RayTracingPipeline>,
    pub scene_acc_buffer: vk::Buffer,
    pub scene_acc_buffer_alloc: SubAllocation,
    pub scene_acc: vk::AccelerationStructureKHR,

    pub rtx_set_layout: vk::DescriptorSetLayout,
    pub rtx_pipe_layout: vk::PipelineLayout,
    pub rtx_pipe: vk::Pipeline,
    pub rtx_pool: vk::DescriptorPool,
    pub rtx_set: vk::DescriptorSet,

    pub rtx_sbt_gen_alloc: SubAllocation,
    pub rtx_sbt_gen: vk::Buffer,
    pub rtx_sbt_gen_addr: vk::DeviceAddress,
    pub rtx_sbt_chit_alloc: SubAllocation,
    pub rtx_sbt_chit: vk::Buffer,
    pub rtx_sbt_chit_addr: vk::DeviceAddress,
    pub rtx_sbt_miss_alloc: SubAllocation,
    pub rtx_sbt_miss: vk::Buffer,
    pub rtx_sbt_miss_addr: vk::DeviceAddress,
}

impl VulkanManager {
    pub fn new(
        engine_info: EngineInfo,
        window: &winit::window::Window,
        max_frames_in_flight: u8,
    ) -> GraphicsResult<Self> {
        let entry = unsafe{ash::Entry::new().map_err(anyhow::Error::from)?};
        let instance = VulkanManager::init_instance(engine_info, &entry, &window)?;

        let debug = DebugMessenger::init(&entry, &instance)?;
        let surface = SurfaceWrapper::init(&window, &entry, &instance);

        let (physical_device, physical_device_properties, _physical_device_features) =
            device::select_physical_device(&instance)?;

        let queue_families = QueueFamilies::init(&instance, physical_device, &surface)?;

        let (logical_device, queues, raytracing_supported) =
            queue::init_device_and_queues(&instance, physical_device, &queue_families)?;

        let logical_device = Rc::new(logical_device);

        let allocator = Rc::new(Allocator::new(instance.clone(), logical_device.clone(), physical_device, raytracing_supported));

        let mut swapchain = SwapchainWrapper::init(
            &instance,
            physical_device,
            &logical_device,
            &surface,
            &queue_families,
            &allocator,
        )?;

        let renderpass_pp_attachments = [vk::AttachmentDescription::builder()
            .format(vk::Format::R16G16B16A16_SFLOAT)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::DONT_CARE)
            .store_op(vk::AttachmentStoreOp::STORE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .build()];
        let renderpass_pp_sub0_refs = [vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build()];
        let renderpass_pp_sub_info = [vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&renderpass_pp_sub0_refs)
            .build()];
        let renderpass_pp_deps = [vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::FRAGMENT_SHADER)
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::SHADER_READ)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .build()];
        let renderpass_pp_info = vk::RenderPassCreateInfo::builder()
            .attachments(&renderpass_pp_attachments)
            .subpasses(&renderpass_pp_sub_info)
            .dependencies(&renderpass_pp_deps)
            .build();
        let renderpass_pp =
            unsafe { logical_device.create_render_pass(&renderpass_pp_info, None)? };

        let renderpass = renderpass::create_deferred_pass(
            vk::Format::R16G16B16A16_SFLOAT,
            vk::Format::D24_UNORM_S8_UINT,
            &logical_device,
        )?;
        swapchain.create_framebuffers(&logical_device, renderpass, renderpass_pp)?;
        let pools = PoolsWrapper::init(&logical_device, &queue_families)?;

        let commandbuffers =
            queue::create_commandbuffers(&logical_device, &pools, max_frames_in_flight as usize)?;

        let uniform_buffer = PerFrameUniformBuffer::<CamData>::new(
            &physical_device_properties,
            &allocator,
            max_frames_in_flight as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
        );

        let desc_layout_frame_data_bindings = [
            // CamData
            vk::DescriptorSetLayoutBinding::builder()
                .binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
                .build(),
            // AlbedoRoughnessTex
            vk::DescriptorSetLayoutBinding::builder()
                .binding(1)
                .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                .build(),
            // NormalMetallicTex
            vk::DescriptorSetLayoutBinding::builder()
                .binding(2)
                .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                .build(),
            // DepthTex
            vk::DescriptorSetLayoutBinding::builder()
                .binding(3)
                .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                .build(),
        ];
        let desc_layout_frame_data_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&desc_layout_frame_data_bindings)
            .build();
        let desc_layout_frame_data = unsafe {
            logical_device.create_descriptor_set_layout(&desc_layout_frame_data_info, None)?
        };

        let pipeline_layout_gpass_push_constants = [vk::PushConstantRange::builder()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .offset(0)
            .size(128)
            .build()];
        let pipeline_layout_gpass_bindings = [desc_layout_frame_data];
        let pipeline_layout_gpass_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&pipeline_layout_gpass_bindings)
            .push_constant_ranges(&pipeline_layout_gpass_push_constants)
            .build();
        let pipeline_layout_gpass =
            unsafe { logical_device.create_pipeline_layout(&pipeline_layout_gpass_info, None)? };

        let pipeline_layout_resolve_pass_push_constants = [vk::PushConstantRange::builder()
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .offset(0)
            .size(32)
            .build()];
        let pipeline_layout_resolve_pass_bindings = [desc_layout_frame_data];
        let pipeline_layout_resolve_pass_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&pipeline_layout_resolve_pass_bindings)
            .push_constant_ranges(&pipeline_layout_resolve_pass_push_constants)
            .build();
        let pipeline_layout_resolve_pass = unsafe {
            logical_device.create_pipeline_layout(&pipeline_layout_resolve_pass_info, None)?
        };

        let descriptor_manager = DescriptorManager::new((*logical_device).clone())?;

        let sem_info = vk::SemaphoreCreateInfo::builder().build();
        let fence_info = vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED)
            .build();

        let mut image_acquire_semaphores = Vec::with_capacity(max_frames_in_flight as usize);
        let mut render_finished_semaphores = Vec::with_capacity(max_frames_in_flight as usize);
        let mut frame_resource_fences = Vec::with_capacity(max_frames_in_flight as usize);

        for _ in 0..max_frames_in_flight {
            image_acquire_semaphores
                .push(unsafe { logical_device.create_semaphore(&sem_info, None)? });
            render_finished_semaphores
                .push(unsafe { logical_device.create_semaphore(&sem_info, None)? });
            frame_resource_fences.push(unsafe { logical_device.create_fence(&fence_info, None)? });
        }

        let sampler_linear_info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .min_lod(0.0)
            .max_lod(vk::LOD_CLAMP_NONE)
            .build();
        let sampler_linear = unsafe { logical_device.create_sampler(&sampler_linear_info, None)? };

        let desc_layout_pp_samplers = [sampler_linear];
        let desc_layout_pp_bindings = [vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .immutable_samplers(&desc_layout_pp_samplers)
            .build()];
        let desc_layout_pp_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&desc_layout_pp_bindings)
            .build();
        let desc_layout_pp =
            unsafe { logical_device.create_descriptor_set_layout(&desc_layout_pp_info, None)? };

        let pipe_layout_pp_sets = [desc_layout_pp];
        let pipe_layout_pp_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&pipe_layout_pp_sets)
            .build();
        let pipe_layout_pp =
            unsafe { logical_device.create_pipeline_layout(&pipe_layout_pp_info, None)? };

        let mut uploader = Uploader::new(
            logical_device.clone(),
            allocator.clone(),
            max_frames_in_flight as u64,
            queue_families.graphics_q_index,
        );

        let rtx_data = if raytracing_supported {
            let rtx_set_bindings = [
                vk::DescriptorSetLayoutBinding::builder()
                    .binding(0)
                    .descriptor_type(vk::DescriptorType::ACCELERATION_STRUCTURE_KHR)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::RAYGEN_KHR)
                    .build(),
                vk::DescriptorSetLayoutBinding::builder()
                    .binding(1)
                    .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::RAYGEN_KHR)
                    .build(),
                vk::DescriptorSetLayoutBinding::builder()
                    .binding(2)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::RAYGEN_KHR)
                    .build(),
            ];
            let rtx_set_layout = unsafe{logical_device.create_descriptor_set_layout(
                &vk::DescriptorSetLayoutCreateInfo::builder()
                    .bindings(&rtx_set_bindings)
                    .build(),
                None
            )}.unwrap();
            
            let rtx_pipe_layout_sets = [ rtx_set_layout ];
            let rtx_pipe_layout = unsafe{logical_device.create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo::builder()
                    .set_layouts(&rtx_pipe_layout_sets)
                    .build(),
                None
            )}.unwrap();

            let rtx_ext = Rc::new(khr::RayTracingPipeline::new(&instance, &logical_device));

            let rtx_pipe = pipeline::create_rtx_pipeline(rtx_pipe_layout, "rtx_test.rgen.spv", "rtx_test.rchit.spv", &logical_device, rtx_ext.clone());
            
            let rtx_pool_sizes = [
                vk::DescriptorPoolSize::builder().ty(vk::DescriptorType::ACCELERATION_STRUCTURE_KHR).descriptor_count(1).build(),
                vk::DescriptorPoolSize::builder().ty(vk::DescriptorType::STORAGE_IMAGE).descriptor_count(1).build(),
                vk::DescriptorPoolSize::builder().ty(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC).descriptor_count(1).build(),
            ];
            let rtx_pool = unsafe{logical_device.create_descriptor_pool(
                &vk::DescriptorPoolCreateInfo::builder()
                    .max_sets(1)
                    .pool_sizes(&rtx_pool_sizes)
                    .build(),
                None
            )}.unwrap();

            let rtx_set = unsafe{logical_device.allocate_descriptor_sets(
                &vk::DescriptorSetAllocateInfo::builder()
                    .descriptor_pool(rtx_pool)
                    .set_layouts(&[rtx_set_layout])
                    .build()
            )}.unwrap()[0];

            let (rtx_sbt_gen, rtx_sbt_gen_alloc) = allocator.create_buffer(32, vk::BufferUsageFlags::SHADER_BINDING_TABLE_KHR | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS | vk::BufferUsageFlags::TRANSFER_DST, gpu_allocator::MemoryLocation::GpuOnly);
            let (rtx_sbt_chit, rtx_sbt_chit_alloc) = allocator.create_buffer(32, vk::BufferUsageFlags::SHADER_BINDING_TABLE_KHR | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS | vk::BufferUsageFlags::TRANSFER_DST, gpu_allocator::MemoryLocation::GpuOnly);
            let (rtx_sbt_miss, rtx_sbt_miss_alloc) = allocator.create_buffer(32, vk::BufferUsageFlags::SHADER_BINDING_TABLE_KHR | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS | vk::BufferUsageFlags::TRANSFER_DST, gpu_allocator::MemoryLocation::GpuOnly);
            let rtx_sbt_gen_addr = unsafe{logical_device.get_buffer_device_address(&vk::BufferDeviceAddressInfo::builder().buffer(rtx_sbt_gen).build())};
            let rtx_sbt_chit_addr = unsafe{logical_device.get_buffer_device_address(&vk::BufferDeviceAddressInfo::builder().buffer(rtx_sbt_chit).build())};
            let rtx_sbt_miss_addr = unsafe{logical_device.get_buffer_device_address(&vk::BufferDeviceAddressInfo::builder().buffer(rtx_sbt_miss).build())};

            let handles = unsafe{rtx_ext.get_ray_tracing_shader_group_handles(rtx_pipe, 0, 2, 64).unwrap()};
            let zero_handle = [0u8; 32];

            uploader.enqueue_buffer_upload(rtx_sbt_gen, 0, &handles[0..32]);
            uploader.enqueue_buffer_upload(rtx_sbt_chit, 0, &handles[32..64]);
            uploader.enqueue_buffer_upload(rtx_sbt_miss, 0, &zero_handle);

            Some(RtxData {
                acc_ext: Rc::new(khr::AccelerationStructure::new(&instance, &logical_device)),
                rtx_ext,
                scene_acc_buffer: vk::Buffer::null(),
                scene_acc_buffer_alloc: SubAllocation::default(),
                scene_acc: vk::AccelerationStructureKHR::null(),
                rtx_set_layout,
                rtx_pipe_layout,
                rtx_pipe,
                rtx_pool,
                rtx_set,

                rtx_sbt_gen_alloc,
                rtx_sbt_gen,
                rtx_sbt_gen_addr,
                rtx_sbt_chit_alloc,
                rtx_sbt_chit,
                rtx_sbt_chit_addr,
                rtx_sbt_miss_alloc,
                rtx_sbt_miss,
                rtx_sbt_miss_addr,
            })
        } else {
            None
        };

        Ok(Self {
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
            pools,
            commandbuffers,
            allocator: std::mem::ManuallyDrop::new(allocator),
            uniform_buffer,
            desc_layout_frame_data,
            pipeline_layout_gpass,
            pipeline_layout_resolve_pass,
            descriptor_manager,
            max_frames_in_flight,
            current_frame_index: 0,
            image_acquire_semaphores,
            render_finished_semaphores,
            frame_resource_fences,
            lighting_pipelines: Vec::new(),
            sampler_linear,
            desc_layout_pp,
            pipe_layout_pp,
            renderpass_pp,
            pp_effects: Vec::new(),
            uploader: std::mem::ManuallyDrop::new(uploader),
            enable_wireframe: false,

            rtx_data,
        })
    }

    pub fn register_lighting_pipeline(&mut self, pipeline: Rc<LightingPipeline>) {
        self.lighting_pipelines.push(pipeline);
    }

    pub fn register_pp_effect(&mut self, pp_effect: Rc<PPEffect>) {
        self.pp_effects.push(pp_effect);
    }

    pub fn get_current_frame_index(&self) -> u8 {
        self.current_frame_index
    }

    fn init_instance(
        engine_info: EngineInfo,
        entry: &ash::Entry,
        window: &winit::window::Window,
    ) -> anyhow::Result<ash::Instance> {
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

        Ok(unsafe { entry.create_instance(&instance_create_info, None) }?)
    }

    pub fn next_frame(&mut self) -> u32 {
        self.current_frame_index = (self.current_frame_index + 1) % self.max_frames_in_flight;

        self.swapchain
            .aquire_next_image(self.image_acquire_semaphores[self.current_frame_index as usize])
    }

    fn set_viewport(&self, commandbuffer: vk::CommandBuffer, width: f32, height: f32) {
        let vp = vk::Viewport {
            x: 0.0,
            y: height,
            width,
            height: -height,
            min_depth: 0.0,
            max_depth: 1.0,
        };
        unsafe {
            self.device.cmd_set_viewport(commandbuffer, 0, &[vp]);
        }
    }

    fn push_constants<T>(
        &self,
        commandbuffer: vk::CommandBuffer,
        layout: vk::PipelineLayout,
        stages: vk::ShaderStageFlags,
        data: &T,
    ) {
        unsafe {
            self.device.cmd_push_constants(
                commandbuffer,
                layout,
                stages,
                0,
                slice::from_raw_parts(data as *const T as *const u8, size_of::<T>()),
            );
        }
    }

    fn blit_image(
        &self,
        commandbuffer: vk::CommandBuffer,
        src: vk::Image,
        dst: vk::Image,
        width: i32,
        height: i32,
    ) {
        let blit = vk::ImageBlit {
            src_subresource: vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            },
            src_offsets: [
                vk::Offset3D { x: 0, y: 0, z: 0 },
                vk::Offset3D {
                    x: width,
                    y: height,
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
                vk::Offset3D { x: 0, y: 0, z: 0 },
                vk::Offset3D {
                    x: width,
                    y: height,
                    z: 1,
                },
            ],
        };
        unsafe {
            self.device.cmd_blit_image(
                commandbuffer,
                src,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                dst,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[blit],
                vk::Filter::NEAREST,
            );
        }
    }

    fn transition_images<const N: usize>(
        device: &ash::Device,
        commandbuffer: vk::CommandBuffer,
        src_stages: vk::PipelineStageFlags,
        dst_stages: vk::PipelineStageFlags,
        transitions: &[ImageTransition; N],
    ) {
        let mut vk_transitions = [vk::ImageMemoryBarrier::builder()
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            })
            .build(); N];

        for (t, vk_t) in transitions.iter().zip(vk_transitions.iter_mut()) {
            vk_t.image = t.image;
            vk_t.old_layout = t.from;
            vk_t.new_layout = t.to;
            vk_t.src_access_mask = t.wait_access;
            vk_t.dst_access_mask = t.dst_access;
        }

        unsafe {
            device.cmd_pipeline_barrier(
                commandbuffer,
                src_stages,
                dst_stages,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &vk_transitions,
            );
        }
    }

    fn begin_renderpass(
        &self,
        commandbuffer: vk::CommandBuffer,
        renderpass: vk::RenderPass,
        framebuffer: vk::Framebuffer,
        clear_values: &[vk::ClearValue],
    ) {
        let info = vk::RenderPassBeginInfo::builder()
            .render_pass(renderpass)
            .framebuffer(framebuffer)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.extent,
            })
            .clear_values(&clear_values);
        unsafe {
            self.device
                .cmd_begin_render_pass(commandbuffer, &info, vk::SubpassContents::INLINE);
        }
    }

    fn build_render_order(models: &[Model]) -> Vec<&Model> {
        let mut res: Vec<&Model> = Vec::with_capacity(models.len());

        for obj in models {
            let mut index = 0;
            for cmp in &res {
                // order: pipeline -> material -> mesh
                if cmp.material.get_pipeline().as_raw() > obj.material.get_pipeline().as_raw() {
                    break;
                }
                if cmp.material.as_ref() as *const Material as *const u8
                    > obj.material.as_ref() as *const Material as *const u8
                {
                    break;
                }
                if cmp.mesh.as_ref() as *const Mesh > obj.mesh.as_ref() as *const Mesh {
                    break;
                }

                index += 1;
            }
            res.insert(index, obj);
        }

        res
    }

    fn render_gpass(
        &mut self,
        commandbuffer: vk::CommandBuffer,
        models: &[&Model],
    ) -> Result<(), vk::Result> {
        let mut last_pipeline = vk::Pipeline::null();
        let mut last_mat: *const u8 = null();
        let mut last_mesh: *const Mesh = null();
        for obj in models {
            unsafe {
                let pipeline = if self.enable_wireframe {
                    obj.material.get_wireframe_pipeline()
                } else {
                    obj.material.get_pipeline()
                };

                if last_pipeline != pipeline {
                    self.device.cmd_bind_pipeline(
                        commandbuffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        pipeline,
                    );
                    self.set_viewport(
                        commandbuffer,
                        self.swapchain.extent.width as f32,
                        self.swapchain.extent.height as f32,
                    );

                    last_pipeline = obj.material.get_pipeline();
                    last_mat = null();
                }

                let mat = obj.material.as_ref() as *const Material as *const u8; // see https://doc.rust-lang.org/std/ptr/fn.eq.html
                if mat != last_mat {
                    let mat_desc_set = self.descriptor_manager.get_descriptor_set(
                        obj.material.get_descriptor_set_layout(),
                        &obj.material.get_descriptor_data(),
                    )?;
                    self.device.cmd_bind_descriptor_sets(
                        commandbuffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        obj.material.get_pipeline_layout(),
                        1,
                        &[mat_desc_set],
                        &[],
                    );

                    last_mat = mat;
                }

                let mesh = obj.mesh.as_ref() as *const Mesh;
                if mesh != last_mesh {
                    self.device.cmd_bind_vertex_buffers(
                        commandbuffer,
                        0,
                        &[obj.mesh.vertex_buffer],
                        &[0],
                    );
                    self.device.cmd_bind_index_buffer(
                        commandbuffer,
                        obj.mesh.index_buffer,
                        0,
                        vk::IndexType::UINT32,
                    );

                    last_mesh = mesh;
                }

                let transform_data = obj.transform.get_transform_data();
                self.push_constants(
                    commandbuffer,
                    obj.material.get_pipeline_layout(),
                    vk::ShaderStageFlags::VERTEX,
                    &transform_data,
                );

                for sm in &obj.mesh.submeshes {
                    self.device
                        .cmd_draw_indexed(commandbuffer, sm.1, 1, sm.0, 0, 0);
                }
            }
        }

        Ok(())
    }

    fn render_resolve_pass(&self, commandbuffer: vk::CommandBuffer, light_manager: &LightManager) {
        for lp in &self.lighting_pipelines {
            unsafe {
                // point lights
                if let Some(point_pipe) = lp.point_pipeline {
                    self.device.cmd_bind_pipeline(
                        commandbuffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        point_pipe,
                    );
                    self.set_viewport(
                        commandbuffer,
                        self.swapchain.extent.width as f32,
                        self.swapchain.extent.height as f32,
                    );

                    for pl in &light_manager.point_lights {
                        self.push_constants(
                            commandbuffer,
                            self.pipeline_layout_resolve_pass,
                            vk::ShaderStageFlags::FRAGMENT,
                            pl,
                        );
                        self.device.cmd_draw(commandbuffer, 6, 1, 0, 0);
                    }
                }

                // directional lights
                if let Some(directional_pipe) = lp.directional_pipeline {
                    self.device.cmd_bind_pipeline(
                        commandbuffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        directional_pipe,
                    );
                    self.set_viewport(
                        commandbuffer,
                        self.swapchain.extent.width as f32,
                        self.swapchain.extent.height as f32,
                    );

                    for dl in &light_manager.directional_lights {
                        self.push_constants(
                            commandbuffer,
                            self.pipeline_layout_resolve_pass,
                            vk::ShaderStageFlags::FRAGMENT,
                            dl,
                        );
                        self.device.cmd_draw(commandbuffer, 6, 1, 0, 0);
                    }
                }

                // ambient
                if let Some(ambient_pipe) = lp.ambient_pipeline {
                    self.device.cmd_bind_pipeline(
                        commandbuffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        ambient_pipe,
                    );
                    self.set_viewport(
                        commandbuffer,
                        self.swapchain.extent.width as f32,
                        self.swapchain.extent.height as f32,
                    );
                    self.device.cmd_draw(commandbuffer, 6, 1, 0, 0);
                }
            }
        }
    }

    fn render_pp(
        &mut self,
        commandbuffer: vk::CommandBuffer,
        swapchain_image_index: usize,
    ) -> Result<(), vk::Result> {
        // resolve image contains finished scene rendering in hdr format
        // for each pp effect:
        //      - transition src image (either resolve_image or g0_image) to SHADER_READONLY_OPTIMAL layout (done by previous pp renderpass and resolve pass)
        //      - transition dst image (either resolve_image or g0_image) to COLOR_ATTACHMENT_OPTIMAL layout (done by renderpass)
        //      - begin pp renderpass with correct framebuffer
        //      - bind pipeline and correct descriptor set
        //      - draw fullscreen quad
        // transition swapchain image to TRANSFER_DST_OPTIMAL
        // transition final dst image (either resolve_image or g0_image) to TRANSFER_SRC_OPTIMAL
        // ImageBlit to swapchain
        // Transition swapchain image to PRESENT_SRC_KHR

        let mut direction = false;

        for effect in &self.pp_effects {
            // begin renderpass
            self.begin_renderpass(
                commandbuffer,
                self.renderpass_pp,
                if !direction {
                    self.swapchain.framebuffer_pp_a
                } else {
                    self.swapchain.framebuffer_pp_b
                },
                &[],
            );

            // bind pp pipeline and descriptor set
            unsafe {
                self.device.cmd_bind_pipeline(
                    commandbuffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    effect.pipeline,
                );
            }

            self.set_viewport(
                commandbuffer,
                self.swapchain.extent.width as f32,
                self.swapchain.extent.height as f32,
            );

            let desc_data = [DescriptorData::ImageSampler {
                image: if !direction {
                    self.swapchain.resolve_imageview
                } else {
                    self.swapchain.g0_imageview
                },
                layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                sampler: vk::Sampler::null(),
            }];
            let desc_set = self
                .descriptor_manager
                .get_descriptor_set(self.desc_layout_pp, &desc_data)?;
            unsafe {
                self.device.cmd_bind_descriptor_sets(
                    commandbuffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.pipe_layout_pp,
                    0,
                    &[desc_set],
                    &[],
                );
            }

            unsafe {
                self.device.cmd_draw(commandbuffer, 6, 1, 0, 0);
                self.device.cmd_end_render_pass(commandbuffer);
            }

            direction = !direction;
        }

        // transition swapchain image to TRANSFER_DST_OPTIMAL and final pp image to TRANSFER_SRC_OPTIMAL
        Self::transition_images(
            &self.device,
            commandbuffer,
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            vk::PipelineStageFlags::TRANSFER,
            &[
                ImageTransition {
                    image: self.swapchain.images[swapchain_image_index],
                    from: vk::ImageLayout::UNDEFINED,
                    to: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    wait_access: vk::AccessFlags::empty(),
                    dst_access: vk::AccessFlags::TRANSFER_WRITE,
                },
                ImageTransition {
                    image: if direction {
                        self.swapchain.g0_image
                    } else {
                        self.swapchain.resolve_image
                    },
                    from: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                    to: vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    wait_access: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                    dst_access: vk::AccessFlags::TRANSFER_READ,
                },
            ],
        );

        // blit pp image to swapchain (automatically converts to sRGB)
        self.blit_image(
            commandbuffer,
            if direction {
                self.swapchain.g0_image
            } else {
                self.swapchain.resolve_image
            },
            self.swapchain.images[swapchain_image_index],
            self.swapchain.extent.width as i32,
            self.swapchain.extent.height as i32,
        );

        // transition swapchain image to PRESENT_SRC_KHR
        Self::transition_images(
            &self.device,
            commandbuffer,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::BOTTOM_OF_PIPE, // can be BOTTOM_OF_PIPE because vkQueuePresentKHR waits for a semaphore
            &[ImageTransition {
                image: self.swapchain.images[swapchain_image_index],
                from: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                to: vk::ImageLayout::PRESENT_SRC_KHR,
                wait_access: vk::AccessFlags::TRANSFER_WRITE,
                dst_access: vk::AccessFlags::empty(), // can be empty because vkQueuePresentKHR waits for a semaphore, which automatically makes all memory writes visible
            }],
        );

        Ok(())
    }

    pub fn update_commandbuffer(
        &mut self,
        swapchain_image_index: usize,
        scene: &Scene,
    ) -> Result<(), vk::Result> {
        let commandbuffer = self.commandbuffers[self.current_frame_index as usize];
        let commandbuffer_begininfo = vk::CommandBufferBeginInfo::builder();
        unsafe {
            self.device
                .begin_command_buffer(commandbuffer, &commandbuffer_begininfo)?;
        }

        if let Some(rtx_data) = &mut self.rtx_data {
            if rtx_data.scene_acc == vk::AccelerationStructureKHR::null() {
                let mut objects = Vec::new();
                for obj in &scene.models {
                    objects.push((
                        obj.mesh.rtx_data.as_ref().unwrap().acc_struct,
                        obj.transform.get_transform_data().model_matrix
                    ));
                }

                let (acc, acc_buffer, acc_alloc) = self.uploader.enqueue_scene_acc_struct_build(rtx_data.acc_ext.clone(), &objects);
                rtx_data.scene_acc = acc;
                rtx_data.scene_acc_buffer = acc_buffer;
                rtx_data.scene_acc_buffer_alloc = acc_alloc;

                unsafe {
                    let mut acc_write = vk::WriteDescriptorSet::builder()
                        .dst_set(rtx_data.rtx_set)
                        .dst_binding(0)
                        .descriptor_type(vk::DescriptorType::ACCELERATION_STRUCTURE_KHR)
                        .build();
                    acc_write.descriptor_count = 1;
                    let accs = [rtx_data.scene_acc];
                    let acc_write_next = vk::WriteDescriptorSetAccelerationStructureKHR::builder()
                        .acceleration_structures(&accs)
                        .build();
                    acc_write.p_next = &acc_write_next as *const _ as *const _;
                    
                    let image_info = [
                        vk::DescriptorImageInfo::builder()
                            .image_view(self.swapchain.g0_imageview)
                            .image_layout(vk::ImageLayout::GENERAL)
                            .build()
                    ];
                    let buffer_info = [
                        vk::DescriptorBufferInfo::builder()
                            .buffer(self.uniform_buffer.get_buffer())
                            .range(self.uniform_buffer.get_size())
                            .build()
                    ];

                    self.device.update_descriptor_sets(&[
                        acc_write,
                        vk::WriteDescriptorSet::builder()
                            .dst_set(rtx_data.rtx_set)
                            .dst_binding(1)
                            .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                            .image_info(&image_info)
                            .build(),
                        vk::WriteDescriptorSet::builder()
                            .dst_set(rtx_data.rtx_set)
                            .dst_binding(2)
                            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
                            .buffer_info(&buffer_info)
                            .build()
                    ],
                &[]);
                }
            }

            Self::transition_images(
                &self.device,
                commandbuffer, vk::PipelineStageFlags::RAY_TRACING_SHADER_KHR, vk::PipelineStageFlags::RAY_TRACING_SHADER_KHR, &[
                ImageTransition {
                    image: self.swapchain.g0_image,
                    from: vk::ImageLayout::UNDEFINED,
                    to: vk::ImageLayout::GENERAL,
                    wait_access: vk::AccessFlags::empty(),
                    dst_access: vk::AccessFlags::SHADER_WRITE,
                },
            ]);
            
            unsafe {
                self.device.cmd_bind_descriptor_sets(commandbuffer, vk::PipelineBindPoint::RAY_TRACING_KHR, rtx_data.rtx_pipe_layout, 0, &[rtx_data.rtx_set], &[self.uniform_buffer.get_offset(self.current_frame_index) as u32]);
                self.device.cmd_bind_pipeline(commandbuffer, vk::PipelineBindPoint::RAY_TRACING_KHR, rtx_data.rtx_pipe);
                rtx_data.rtx_ext.cmd_trace_rays(
                    commandbuffer,
                    &vk::StridedDeviceAddressRegionKHR::builder()
                        .device_address(rtx_data.rtx_sbt_gen_addr)
                        .stride(32)
                        .size(32)
                        .build(),
                    &vk::StridedDeviceAddressRegionKHR::builder()
                        .device_address(rtx_data.rtx_sbt_miss_addr)
                        .stride(32)
                        .size(32)
                        .build(),
                    &vk::StridedDeviceAddressRegionKHR::builder()
                        .device_address(rtx_data.rtx_sbt_chit_addr)
                        .stride(32)
                        .size(32)
                        .build(),
                    &vk::StridedDeviceAddressRegionKHR::builder()
                        .stride(0)
                        .size(0)
                        .build(),
                    self.swapchain.extent.width,
                    self.swapchain.extent.height,
                    1
                );
            }

            Self::transition_images(
                &self.device,
                commandbuffer, vk::PipelineStageFlags::RAY_TRACING_SHADER_KHR, vk::PipelineStageFlags::TRANSFER, &[
                ImageTransition {
                    image: self.swapchain.g0_image,
                    from: vk::ImageLayout::GENERAL,
                    to: vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    wait_access: vk::AccessFlags::SHADER_WRITE,
                    dst_access: vk::AccessFlags::TRANSFER_READ,
                },
                ImageTransition {
                    image: self.swapchain.images[swapchain_image_index],
                    from: vk::ImageLayout::UNDEFINED,
                    to: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    wait_access: vk::AccessFlags::empty(),
                    dst_access: vk::AccessFlags::TRANSFER_WRITE,
                },
            ]);

            self.blit_image(
                commandbuffer,
                self.swapchain.g0_image,
                self.swapchain.images[swapchain_image_index],
                self.swapchain.extent.width as i32,
                self.swapchain.extent.height as i32
            );

            Self::transition_images(
                &self.device,
                commandbuffer, vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::BOTTOM_OF_PIPE, &[
                ImageTransition {
                    image: self.swapchain.images[swapchain_image_index],
                    from: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    to: vk::ImageLayout::PRESENT_SRC_KHR,
                    wait_access: vk::AccessFlags::TRANSFER_WRITE,
                    dst_access: vk::AccessFlags::empty(),
                },
            ]);
        } else {
            self.begin_renderpass(
                commandbuffer,
                self.renderpass,
                self.swapchain.framebuffer_deferred,
                &[
                    vk::ClearValue {
                        color: vk::ClearColorValue {
                            float32: [0.2, 0.2, 0.2, 0.0],
                        },
                    },
                    vk::ClearValue {
                        depth_stencil: vk::ClearDepthStencilValue {
                            depth: 1.0,
                            stencil: 0,
                        },
                    },
                ],
            );
    
            let desc_values_frame_data = [
                DescriptorData::DynamicUniformBuffer {
                    buffer: self.uniform_buffer.get_buffer(),
                    offset: 0,
                    size: self.uniform_buffer.get_size(),
                },
                DescriptorData::InputAttachment {
                    image: self.swapchain.g0_imageview,
                    layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                },
                DescriptorData::InputAttachment {
                    image: self.swapchain.g1_imageview,
                    layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                },
                DescriptorData::InputAttachment {
                    image: self.swapchain.depth_imageview_depth_only,
                    layout: vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL,
                },
            ];
            let desc_set_camera = self
                .descriptor_manager
                .get_descriptor_set(self.desc_layout_frame_data, &desc_values_frame_data)?;
    
            unsafe {
                self.device.cmd_bind_descriptor_sets(
                    commandbuffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.pipeline_layout_gpass,
                    0,
                    &[desc_set_camera],
                    &[self.uniform_buffer.get_offset(self.current_frame_index) as u32],
                );
            }
    
            let render_map = Self::build_render_order(&scene.models);
            self.render_gpass(commandbuffer, &render_map)?;
    
            unsafe {
                self.device
                    .cmd_next_subpass(commandbuffer, vk::SubpassContents::INLINE);
    
                self.device.cmd_bind_descriptor_sets(
                    commandbuffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.pipeline_layout_resolve_pass,
                    0,
                    &[desc_set_camera],
                    &[self.uniform_buffer.get_offset(self.current_frame_index) as u32],
                );
            }
    
            self.render_resolve_pass(commandbuffer, &scene.light_manager);
    
            unsafe {
                self.device.cmd_end_render_pass(commandbuffer);
                self.render_pp(commandbuffer, swapchain_image_index)?;
            }
        }

        unsafe {
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
            .create_framebuffers(&self.device, self.renderpass, self.renderpass_pp)?;
        Ok(())
    }

    pub fn wait_for_fence(&mut self) {
        unsafe {
            self.device
                .wait_for_fences(
                    &[self.frame_resource_fences[self.current_frame_index as usize]],
                    true,
                    std::u64::MAX,
                )
                .expect("fence-waiting");
            self.device
                .reset_fences(&[self.frame_resource_fences[self.current_frame_index as usize]])
                .expect("resetting fences");
        }

        self.descriptor_manager.next_frame();
        self.uploader.submit_uploads(self.queues.graphics_queue);
    }

    /// submits queued commands
    pub fn submit(&self) {
        let semaphores_available =
            [self.image_acquire_semaphores[self.current_frame_index as usize]];
        let waiting_stages = [vk::PipelineStageFlags::TRANSFER]; // wait for image acquisition before blitting the final image to the swapchain
        let semaphores_finished =
            [self.render_finished_semaphores[self.current_frame_index as usize]];
        let commandbuffers = [self.commandbuffers[self.current_frame_index as usize]];
        let submit_info = [vk::SubmitInfo::builder()
            .wait_semaphores(&semaphores_available)
            .wait_dst_stage_mask(&waiting_stages)
            .command_buffers(&commandbuffers)
            .signal_semaphores(&semaphores_finished)
            .build()];
        unsafe {
            self.device
                .queue_submit(
                    self.queues.graphics_queue,
                    &submit_info,
                    self.frame_resource_fences[self.current_frame_index as usize],
                )
                .expect("queue submission");
        };
    }

    /// add present command to queue
    pub fn present(&mut self, image_index: u32) {
        let swapchains = [self.swapchain.swapchain];
        let indices = [image_index];
        let wait_semaphores = [self.render_finished_semaphores[self.current_frame_index as usize]];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&wait_semaphores)
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
    }

    pub fn wait_idle(&self) {
        unsafe {
            self.device
                .device_wait_idle()
                .expect("device_wait_idle() failed");
        }
    }
}

impl Drop for VulkanManager {
    fn drop(&mut self) {
        unsafe {
            self.device
                .device_wait_idle()
                .expect("something wrong while waiting");

            self.lighting_pipelines.clear();
            self.pp_effects.clear();

            for s in &self.image_acquire_semaphores {
                self.device.destroy_semaphore(*s, None);
            }
            for s in &self.render_finished_semaphores {
                self.device.destroy_semaphore(*s, None);
            }
            for f in &self.frame_resource_fences {
                self.device.destroy_fence(*f, None);
            }

            self.descriptor_manager.destroy();

            self.uploader.destroy();
            std::mem::ManuallyDrop::drop(&mut self.uploader);

            self.uniform_buffer.destroy(&self.allocator);

            self.pools.cleanup(&self.device);

            self.device.destroy_render_pass(self.renderpass, None);
            self.device.destroy_render_pass(self.renderpass_pp, None);
            // --segfault
            self.swapchain.cleanup(&self.device, &self.allocator);

            self.device.destroy_sampler(self.sampler_linear, None);

            self.device
                .destroy_descriptor_set_layout(self.desc_layout_frame_data, None);
            self.device
                .destroy_descriptor_set_layout(self.desc_layout_pp, None);

            self.device
                .destroy_pipeline_layout(self.pipeline_layout_gpass, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout_resolve_pass, None);
            self.device
                .destroy_pipeline_layout(self.pipe_layout_pp, None);

            std::mem::ManuallyDrop::drop(&mut self.allocator);

            self.device.destroy_device(None);
            // --segfault
            std::mem::ManuallyDrop::drop(&mut self.surface);
            std::mem::ManuallyDrop::drop(&mut self.debug);
            self.instance.destroy_instance(None)
        };
    }
}

struct ImageTransition {
    image: vk::Image,
    from: vk::ImageLayout,
    to: vk::ImageLayout,
    wait_access: vk::AccessFlags,
    dst_access: vk::AccessFlags,
}
