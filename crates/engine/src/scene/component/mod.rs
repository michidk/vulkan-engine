pub mod renderer;
pub mod test;

use std::{
    cell::RefCell,
    fmt::Debug,
    rc::{Rc, Weak},
};

use super::{entity::Entity, Scene};

pub trait Component: Debug {
    fn attach(&mut self, scene: Rc<RefCell<Scene>>, entity: Weak<RefCell<Entity>>);
    fn load(&mut self);
    fn start(&self);
    fn update(&self);
}
