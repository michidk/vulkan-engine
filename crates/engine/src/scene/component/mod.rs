pub mod renderer;

use std::{fmt::Debug, rc::Rc};

use super::{entity::Entity, model::Model, transform::TransformData};

pub trait Component: Debug {
    fn create(entity: &Rc<Entity>) -> Rc<Self>
    where
        Self: Sized;

    fn load(&self);
    fn start(&self);
    fn update(&self, delta: f32);

    fn render(&self, _models: &mut Vec<(TransformData, Rc<Model>)>) {}
}
