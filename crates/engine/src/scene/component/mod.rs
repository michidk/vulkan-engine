pub mod renderer;

use std::{
    fmt::Debug,
    rc::Weak,
};

use super::{entity::Entity, Scene};

pub trait Component: Debug {
    fn attach(&self, scene: Weak<Scene>, entity: Weak<Entity>);
    fn load(&self);
    fn start(&self);
    fn update(&self);
}
