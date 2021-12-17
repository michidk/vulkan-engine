pub mod renderer;

use std::{
    fmt::Debug,
    rc::{Rc, Weak},
};

use super::{entity::Entity, model::Model, transform::TransformData, Scene};

pub trait Component: Debug {
    fn attach(&self, scene: Weak<Scene>, entity: Weak<Entity>);
    fn load(&self);
    fn start(&self);
    fn update(&self, delta: f32);

    fn render(&self, _models: &mut Vec<(TransformData, Rc<Model>)>) {}
}
