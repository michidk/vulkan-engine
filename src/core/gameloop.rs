use crate::{scene::Scene, vulkan::VulkanManager};

pub struct GameLoop {}

impl GameLoop {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) fn init(&self) {}

    // todo: implement Update, Render traits and then create type def of combined type; then have a list of them in SceneManager, and call update for all of them
    pub(crate) fn update(&self, vulkan_manager: &mut VulkanManager, scene: &Scene) {
        let vk = vulkan_manager;
        self.render(vk, scene);
    }

    // TODO: @ROB make copies of instance buffer to have exactly one buffer copy per image
    fn render(&self, vk: &mut VulkanManager, scene: &Scene) {
        // prepare for render
        let image_index = vk.swapchain.aquire_next_image();
        vk.wait_for_fence();

        vk.descriptor_manager.next_frame();

        scene
            .light_manager
            .update_buffer(&vk.allocator, &mut vk.light_buffer)
            .expect("Something went wrong when updating light");

        vk.update_commandbuffer(image_index as usize)
            .expect("updating the command buffer");

        // finanlize renderpass
        let semaphores_finished = vk.submit(image_index);
        vk.present(image_index, &semaphores_finished);
        vk.swapchain.swap();
    }
}
