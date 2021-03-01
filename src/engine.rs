use crate::{
    core::{gameloop::GameLoop, window},
    scene::Scene,
    vulkan::VulkanManager,
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Info {
    pub window_info: window::Info,
    pub app_name: &'static str,
}

pub struct EngineInit {
    pub eventloop: winit::event_loop::EventLoop<()>,
    pub engine: Engine,
}

impl EngineInit {
    pub fn new(info: Info) -> Result<Self, Box<dyn std::error::Error>> {
        let scene = Scene::new();

        let eventloop = winit::event_loop::EventLoop::new();
        let window = info.window_info.into_window(&eventloop)?;

        let vulkan_manager = VulkanManager::new(info, window, 3)?;
        let gameloop = GameLoop::new();

        Ok(Self {
            eventloop,
            engine: Engine {
                info,
                gameloop,
                scene,
                vulkan_manager,
            },
        })
    }

    pub fn start(self) -> ! {
        window::start(self);
    }
}

pub struct Engine {
    pub info: Info,
    pub gameloop: GameLoop,
    pub scene: Scene,
    pub vulkan_manager: VulkanManager,
}

impl Engine {
    pub fn init(&self) {
        self.gameloop.init();
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.vulkan_manager.wait_idle();
    }
}
