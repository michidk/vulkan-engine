use std::{cell::RefCell, rc::Rc};

use winit::event::VirtualKeyCode;

use crate::{scene::Scene, vulkan::VulkanManager};

use super::input::Input;

pub struct GameLoop {
    input: Rc<RefCell<Input>>,
}

impl GameLoop {
    pub(crate) fn new(input: Rc<RefCell<Input>>) -> Self {
        Self { input }
    }

    pub(crate) fn init(&self) {}

    // TODO: implement Update, Render traits and then create type def of combined type; then have a list of them in SceneManager, and call update for all of them
    pub(crate) fn update(&self, _vulkan_manager: &mut VulkanManager, _scene: &Scene) {
        if self.input.borrow().button_was_down(VirtualKeyCode::W) {
            println!("key was pressed!")
        }
        // let key = VirtualKeyCode::W;
        let _input = self.input.borrow();
        // if input.state.key_held[key as usize] {
        //     println!(
        //         "{:?} - {:?}",
        //         input.state_prev.key_held[key as usize], input.state.key_held[key as usize]
        //     );
        // }

        // println!("{:?}", input.state.mouse_position);
    }
}
