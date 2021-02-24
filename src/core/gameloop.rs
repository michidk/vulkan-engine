use ash::{version::DeviceV1_0, vk};

use crate::{
    scene::{
        camera::{self, Camera},
        Scene,
    },
    vulkan::manager::VulkanManager,
};

pub struct GameLoop {}

impl GameLoop {
    pub(crate) fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {})
    }

    pub(crate) fn init(&self) {}

    pub(crate) fn update(&self, vulkan_manager: &mut VulkanManager, scene: &Scene) {
        let mut vk = vulkan_manager;

        scene.light_manager.update_buffer(
            &vk.device,
            &vk.allocator,
            &mut vk.light_buffer,
            &mut vk.descriptor_sets_light,
        );
        let (image_index, _) = unsafe {
            vk.swapchain
                .swapchain_loader
                .acquire_next_image(
                    vk.swapchain.swapchain,
                    std::u64::MAX,
                    vk.swapchain.image_available[vk.swapchain.current_image],
                    vk::Fence::null(),
                )
                .expect("image acquisition trouble")
        };
        unsafe {
            &vk.device
                .wait_for_fences(
                    &[vk.swapchain.may_begin_drawing[vk.swapchain.current_image]],
                    true,
                    std::u64::MAX,
                )
                .expect("fence-waiting");
            &vk.device
                .reset_fences(&[vk.swapchain.may_begin_drawing[vk.swapchain.current_image]])
                .expect("resetting fences");
        }
        // camera.update_buffer(&vk.allocator, &mut &vk.uniform_buffer);
        for m in &mut vk.models {
            m.update_instance_buffer(&vk.allocator).unwrap();
        }
        &vk.update_commandbuffer(image_index as usize)
            .expect("updating the command buffer");

        let semaphores_available = [vk.swapchain.image_available[vk.swapchain.current_image]];
        let waiting_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let semaphores_finished = [vk.swapchain.rendering_finished[vk.swapchain.current_image]];
        let commandbuffers = [vk.commandbuffers[image_index as usize]];
        let submit_info = [vk::SubmitInfo::builder()
            .wait_semaphores(&semaphores_available)
            .wait_dst_stage_mask(&waiting_stages)
            .command_buffers(&commandbuffers)
            .signal_semaphores(&semaphores_finished)
            .build()];
        unsafe {
            &vk.device
                .queue_submit(
                    vk.queues.graphics_queue,
                    &submit_info,
                    vk.swapchain.may_begin_drawing[vk.swapchain.current_image],
                )
                .expect("queue submission");
        };
        let swapchains = [vk.swapchain.swapchain];
        let indices = [image_index];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&semaphores_finished)
            .swapchains(&swapchains)
            .image_indices(&indices);
        unsafe {
            match &vk
                .swapchain
                .swapchain_loader
                .queue_present(vk.queues.graphics_queue, &present_info)
            {
                Ok(..) => {}
                Err(ash::vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    &vk.recreate_swapchain().expect("swapchain recreation");
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
        vk.swapchain.current_image =
            (vk.swapchain.current_image + 1) % vk.swapchain.amount_of_images as usize;
    }
}
