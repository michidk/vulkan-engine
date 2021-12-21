use std::{cell::RefCell, rc::Rc};
use winit::event::VirtualKeyCode;

use super::input::Input;
use crate::{scene::Scene, vulkan::VulkanManager};

pub struct GameLoop {
    input: Rc<RefCell<Input>>,
}

impl GameLoop {
    pub(crate) fn new(input: Rc<RefCell<Input>>) -> Self {
        Self { input }
    }

    pub(crate) fn init(&self) {}

    // TODO: implement Update, Render traits and then create type def of combined type; then have a list of them in SceneManager, and call update for all of them
    pub(crate) fn update(&self, vulkan_manager: &mut VulkanManager, scene: &Scene, delta: f32) {
        let input = self.input.borrow();

        if input.get_button_was_down(VirtualKeyCode::F) {
            vulkan_manager.enable_wireframe = !vulkan_manager.enable_wireframe;
        }

        scene.update(&input, delta);
    }
}
