use std::{cell::RefCell, rc::Rc};

use crate::{
    core::{camera::Camera, gameloop::GameLoop, input::Input, window},
    scene::Scene,
    vulkan::VulkanManager,
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct EngineInfo {
    pub window_info: window::WindowInfo,
    pub app_name: &'static str,
}

pub struct EngineInit {
    pub eventloop: winit::event_loop::EventLoop<()>,
    pub engine: Engine,
}

impl EngineInit {
    pub fn new(info: EngineInfo, camera: Camera) -> Result<Self, Box<dyn std::error::Error>> {
        let scene = Scene::new();
        let eventloop = winit::event_loop::EventLoop::new();
        let window = info.window_info.into_window(&eventloop)?;

        let vulkan_manager = VulkanManager::new(info, window, 3)?;
        let input = Rc::new(RefCell::new(Input::new()));
        let gameloop = GameLoop::new(input.clone());

        Ok(Self {
            eventloop,
            engine: Engine {
                info,
                gameloop,
                input,
                scene,
                camera,
                vulkan_manager,
            },
        })
    }

    pub fn start(self) -> ! {
        window::start(self);
    }
}

pub struct Engine {
    pub info: EngineInfo,
    pub gameloop: GameLoop,
    pub input: Rc<RefCell<Input>>,
    pub scene: Scene,
    pub camera: Camera,
    pub vulkan_manager: VulkanManager,
}

impl Engine {
    pub fn init(&self) {
        self.gameloop.init();
    }

    pub(crate) fn render(&mut self) {
        let vk = &mut self.vulkan_manager;

        // move cam
        self.camera.movement(&self.input.borrow());

        // prepare for render
        let image_index = vk.next_frame();
        vk.wait_for_fence();

        self.camera.update_buffer(
            &vk.allocator,
            &mut vk.uniform_buffer,
            vk.current_frame_index,
        );

        vk.update_commandbuffer(image_index as usize, &self.scene)
            .expect("updating the command buffer");

        // finanlize renderpass
        vk.submit();
        vk.present(image_index);
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.vulkan_manager.wait_idle();
    }
}
