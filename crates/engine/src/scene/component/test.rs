use std::{cell::RefCell, rc::Rc};

use crate::scene::{entity::Entity, Scene};

use super::Component;

// #[derive(Debug)]
// pub struct TestComponent {
//     pub value: u32
// }

// impl TestComponent {
//     pub fn new() -> Self {
//         Self {
//             value: 123u32
//         }
//     }
// }

// impl Component for TestComponent {
//     fn attach(&self, scene: Option<Rc<Scene>>, entity: Option<Rc<RefCell<Entity>>>) {
//         println!("TestComponent::attach()");
//     }
//     fn load(&self) {
//         println!("Load Ref - Value: {}", &self.value);
//     }
//     fn start(&self) {
//         println!("Start");
//     }
//     fn update(&self) {
//         println!("Update");
//     }
// }
