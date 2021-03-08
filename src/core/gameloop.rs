use crystal::prelude::Vec3;

use crate::{
    scene::{camera::Camera, Scene},
    vulkan::VulkanManager,
};

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
        let image_index = vk.next_frame();
        vk.wait_for_fence();

        // TODO: move out of gameloop
        let camera = Camera::builder()
            .position(Vec3::new(0.0, 0.0, 0.0))
            .aspect(vk.swapchain.extent.width as f32 / vk.swapchain.extent.height as f32)
            .build();

        camera.update_buffer(
            &vk.allocator,
            &mut vk.uniform_buffer,
            vk.current_frame_index,
        );

        vk.update_commandbuffer(image_index as usize, scene)
            .expect("updating the command buffer");

        // finanlize renderpass
        vk.submit();
        vk.present(image_index);
    }
}
