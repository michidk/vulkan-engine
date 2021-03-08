pub(crate) mod buffer;
mod debug;
pub mod descriptor_manager;
mod device;
pub mod error;
mod pipeline;
mod queue;
mod renderpass;
mod surface;
mod swapchain;
pub mod lighting_pipeline;

use std::{ffi::CString, mem::size_of, rc::Rc, slice};

use ash::{
    extensions::ext,
    version::{DeviceV1_0, EntryV1_0, InstanceV1_0},
    vk,
};

use crate::{assets::shader, engine::Info, scene::{Scene, camera, light::{DirectionalLight, LightManager, PointLight}, transform::TransformData}};

use self::{buffer::{BufferWrapper, PerFrameUniformBuffer, VulkanBuffer}, debug::DebugMessenger, descriptor_manager::{DescriptorData, DescriptorManager}, lighting_pipeline::LightingPipeline, queue::{PoolsWrapper, QueueFamilies, Queues}, surface::SurfaceWrapper, swapchain::SwapchainWrapper};

pub struct VulkanManager {
    pub window: winit::window::Window,
    #[allow(dead_code)]
    entry: ash::Entry,
    instance: ash::Instance,
    pub allocator: std::mem::ManuallyDrop<Rc<vk_mem::Allocator>>,
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
}

impl VulkanManager {
    pub fn new(
        engine_info: Info,
        window: winit::window::Window,
        max_frames_in_flight: u8,
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
        let pools = PoolsWrapper::init(&logical_device, &queue_families)?;

        let commandbuffers =
            queue::create_commandbuffers(&logical_device, &pools, max_frames_in_flight as usize)?;

        let uniform_buffer = PerFrameUniformBuffer::new(
            &physical_device_properties,
            &allocator,
            max_frames_in_flight as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
        )?;

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

        let pipeline_layout_gpass_push_constants = [
            vk::PushConstantRange::builder()
                .stage_flags(vk::ShaderStageFlags::VERTEX)
                .offset(0)
                .size(128)
                .build()
        ];
        let pipeline_layout_gpass_bindings = [desc_layout_frame_data];
        let pipeline_layout_gpass_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&pipeline_layout_gpass_bindings)
            .push_constant_ranges(&pipeline_layout_gpass_push_constants)
            .build();
        let pipeline_layout_gpass = unsafe {
            logical_device.create_pipeline_layout(&pipeline_layout_gpass_info, None)?
        };

        let pipeline_layout_resolve_pass_push_constants = [
            vk::PushConstantRange::builder()
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                .offset(0)
                .size(32)
                .build()
        ];
        let pipeline_layout_resolve_pass_bindings = [desc_layout_frame_data];
        let pipeline_layout_resolve_pass_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&pipeline_layout_resolve_pass_bindings)
            .push_constant_ranges(&pipeline_layout_resolve_pass_push_constants)
            .build();
        let pipeline_layout_resolve_pass = unsafe {
            logical_device.create_pipeline_layout(&pipeline_layout_resolve_pass_info, None)?
        };

        let descriptor_manager = DescriptorManager::new(logical_device.clone())?;

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
            device: Rc::new(logical_device),
            swapchain,
            renderpass,
            pools,
            commandbuffers,
            allocator: std::mem::ManuallyDrop::new(Rc::new(allocator)),
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
            lighting_pipelines: Vec::new()
        })
    }

    pub fn register_lighting_pipeline(&mut self, pipeline: Rc<LightingPipeline>) {
        self.lighting_pipelines.push(pipeline);
    }

    pub fn get_current_frame_index(&self) -> u8 {
        self.current_frame_index
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

    pub fn next_frame(&mut self) -> u32 {
        self.current_frame_index = (self.current_frame_index + 1) % self.max_frames_in_flight;
        self.descriptor_manager.next_frame();

        self.swapchain
            .aquire_next_image(self.image_acquire_semaphores[self.current_frame_index as usize])
    }

    pub fn update_commandbuffer(
        &mut self,
        swapchain_image_index: usize,
        scene: &Scene
    ) -> Result<(), vk::Result> {
        let commandbuffer = self.commandbuffers[self.current_frame_index as usize];
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
            .framebuffer(self.swapchain.framebuffers[swapchain_image_index])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.extent,
            })
            .clear_values(&clearvalues);

        let desc_values_frame_data = [
            DescriptorData::DynamicUniformBuffer {
                buffer: self.uniform_buffer.get_buffer(),
                offset: 0,
                size: self.uniform_buffer.get_size(),
            },
            DescriptorData::InputAttachment {
                image: self.swapchain.g0_imageview,
                layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
            },
            DescriptorData::InputAttachment {
                image: self.swapchain.g1_imageview,
                layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
            },
            DescriptorData::InputAttachment {
                image: self.swapchain.depth_imageview_depth_only,
                layout: vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL
            },
        ];
        let desc_set_camera = self
            .descriptor_manager
            .get_descriptor_set(self.desc_layout_frame_data, &desc_values_frame_data)?;

        unsafe {
            self.device.cmd_begin_render_pass(
                commandbuffer,
                &renderpass_begininfo,
                vk::SubpassContents::INLINE,
            );
        }

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

        for obj in &scene.models {
            unsafe {
                self.device.cmd_bind_pipeline(
                    commandbuffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    obj.material.get_pipeline(),
                );

                let vp = vk::Viewport {
                    x: 0.0,
                    y: self.swapchain.extent.height as f32,
                    width: self.swapchain.extent.width as f32,
                    height: -(self.swapchain.extent.height as f32),
                    min_depth: 0.0,
                    max_depth: 1.0,
                };
                self.device.cmd_set_viewport(commandbuffer, 0, &[vp]);

                let mat_desc_data = obj.material.get_descriptor_data();
                let mat_desc_set = self
                    .descriptor_manager
                    .get_descriptor_set(obj.material.get_descriptor_set_layout(), mat_desc_data)?;
                self.device.cmd_bind_descriptor_sets(
                    commandbuffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    obj.material.get_pipeline_layout(),
                    1,
                    &[mat_desc_set],
                    &[],
                );

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

                let transform_data = obj.transform.get_transform_data();
                self.device.cmd_push_constants(
                    commandbuffer, 
                    obj.material.get_pipeline_layout(),
                    vk::ShaderStageFlags::VERTEX,
                    0,
                    slice::from_raw_parts(&transform_data as *const TransformData as *const u8, size_of::<TransformData>())
                );

                for sm in &obj.mesh.submeshes {
                    self.device
                        .cmd_draw_indexed(commandbuffer, sm.1, 1, sm.0, 0, 0);
                }
            }
        }

        unsafe {
            self.device.cmd_next_subpass(commandbuffer, vk::SubpassContents::INLINE);
        }

        for lp in &self.lighting_pipelines {
            unsafe {
                // point lights
                if let Some(point_pipe) = lp.point_pipeline {
                    self.device.cmd_bind_pipeline(commandbuffer, vk::PipelineBindPoint::GRAPHICS, point_pipe);
            
                    let vp = vk::Viewport {
                        x: 0.0,
                        y: self.swapchain.extent.height as f32,
                        width: self.swapchain.extent.width as f32,
                        height: -(self.swapchain.extent.height as f32),
                        min_depth: 0.0,
                        max_depth: 1.0,
                    };
                    self.device.cmd_set_viewport(commandbuffer, 0, &[vp]);
    
                    for pl in &scene.light_manager.point_lights {
                        self.device.cmd_push_constants(
                            commandbuffer, 
                            self.pipeline_layout_resolve_pass, 
                            vk::ShaderStageFlags::FRAGMENT,
                            0,
                            slice::from_raw_parts(pl as *const PointLight as *const u8, size_of::<PointLight>())
                        );
                        self.device.cmd_draw(commandbuffer, 6, 1, 0, 0);
                    }
                }

                // directional lights
                if let Some(directional_pipe) = lp.directional_pipeline {
                    self.device.cmd_bind_pipeline(commandbuffer, vk::PipelineBindPoint::GRAPHICS, directional_pipe);
            
                    let vp = vk::Viewport {
                        x: 0.0,
                        y: self.swapchain.extent.height as f32,
                        width: self.swapchain.extent.width as f32,
                        height: -(self.swapchain.extent.height as f32),
                        min_depth: 0.0,
                        max_depth: 1.0,
                    };
                    self.device.cmd_set_viewport(commandbuffer, 0, &[vp]);

                    for dl in &scene.light_manager.directional_lights {
                        self.device.cmd_push_constants(
                            commandbuffer, 
                            self.pipeline_layout_resolve_pass, 
                            vk::ShaderStageFlags::FRAGMENT,
                            0,
                            slice::from_raw_parts(dl as *const DirectionalLight as *const u8, size_of::<DirectionalLight>())
                        );
                        self.device.cmd_draw(commandbuffer, 6, 1, 0, 0);
                    }
                }

                // ambient
                if let Some(ambient_pipe) = lp.ambient_pipeline {
                    self.device.cmd_bind_pipeline(commandbuffer, vk::PipelineBindPoint::GRAPHICS, ambient_pipe);
            
                    let vp = vk::Viewport {
                        x: 0.0,
                        y: self.swapchain.extent.height as f32,
                        width: self.swapchain.extent.width as f32,
                        height: -(self.swapchain.extent.height as f32),
                        min_depth: 0.0,
                        max_depth: 1.0,
                    };
                    self.device.cmd_set_viewport(commandbuffer, 0, &[vp]);
                    self.device.cmd_draw(commandbuffer, 6, 1, 0, 0);
                }
            }
        }
        
        unsafe {
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
        Ok(())
    }

    pub fn wait_for_fence(&self) {
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
    }

    /// submits queued commands
    pub fn submit(&self) {
        let semaphores_available =
            [self.image_acquire_semaphores[self.current_frame_index as usize]];
        let waiting_stages = [vk::PipelineStageFlags::TOP_OF_PIPE];
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

            self.uniform_buffer.destroy(&self.allocator);

            self.pools.cleanup(&self.device);
            
            self.device.destroy_render_pass(self.renderpass, None);
            // --segfault
            self.swapchain.cleanup(&self.device, &self.allocator);

            self.device
                .destroy_descriptor_set_layout(self.desc_layout_frame_data, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout_gpass, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout_resolve_pass, None);

            std::mem::ManuallyDrop::drop(&mut self.allocator);

            self.device.destroy_device(None);
            // --segfault
            std::mem::ManuallyDrop::drop(&mut self.surface);
            std::mem::ManuallyDrop::drop(&mut self.debug);
            self.instance.destroy_instance(None)
        };
    }
}
