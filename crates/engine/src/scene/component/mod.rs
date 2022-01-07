pub mod camera_component;
pub mod debug_movement_component;
pub mod renderer;

use std::{fmt::Debug, rc::Rc};

use crate::core::input::Input;

use super::{entity::Entity, model::Model, transform::TransformData};

pub trait Component: Debug {
    fn create(entity: &Rc<Entity>) -> Rc<Self>
    where
        Self: Sized;

    fn inspector_name(&self) -> &'static str;
    fn render_inspector(&self, _ui: &mut egui::Ui) {}

    fn load(&self);
    fn start(&self);

    fn update(&self, _input: &Input, _delta: f32) {}
    fn render(&self, _models: &mut Vec<(TransformData, Rc<Model>)>) {}
}
