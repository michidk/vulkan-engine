pub mod renderer;

use std::{
    cell::RefCell,
    fmt::Debug,
    rc::Weak,
};

use super::{entity::Entity, Scene};

pub trait Component: Debug {
    fn attach(&mut self, scene: Weak<Scene>, entity: Weak<RefCell<Entity>>);
    fn load(&mut self);
    fn start(&self);
    fn update(&self);
}
