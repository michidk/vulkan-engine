pub mod camera_component;
pub mod debug_movement_component;
pub mod light_component;
pub mod renderer;

use std::{cell::Cell, fmt::Debug, rc::Rc};

use egui::TextEdit;

use crate::core::input::Input;

use super::{entity::Entity, light::Light, model::Model, transform::TransformData};

pub trait Component: Debug + ComponentName + ComponentInspector {
    fn create(entity: &Rc<Entity>) -> Rc<Self>
    where
        Self: Sized;

    fn load(&self) {}
    fn start(&self) {}

    fn update(&self, _input: &Input, _delta: f32) {}
    fn render(&self, _models: &mut Vec<(TransformData, Rc<Model>)>, _lights: &mut Vec<Light>) {}
}

pub trait ComponentName {
    fn component_name(&self) -> &'static str;
    fn static_component_name() -> &'static str
    where
        Self: Sized;
}

pub trait ComponentInspector {
    fn render_inspector(&self, ui: &mut egui::Ui);
}

pub trait InspectableValueRead {
    type Value;

    fn get_value(&self) -> Self::Value;
}

pub trait InspectableValue: InspectableValueRead {
    fn set_value(&self, val: Self::Value);
}

pub trait InspectableRendererRead<V> {
    fn render_value<Value: InspectableValueRead<Value = V>>(self, ui: &mut egui::Ui, val: &Value);
}

pub trait InspectableRenderer<V> {
    fn render_value<Value: InspectableValue<Value = V>>(self, ui: &mut egui::Ui, val: &Value);
}

impl InspectableValueRead for f32 {
    type Value = f32;

    fn get_value(&self) -> Self::Value {
        *self
    }
}

impl<T: Copy> InspectableValueRead for Cell<T> {
    type Value = T;

    fn get_value(&self) -> Self::Value {
        self.get()
    }
}

impl<T: Copy> InspectableValue for Cell<T> {
    fn set_value(&self, val: Self::Value) {
        self.set(val);
    }
}

pub struct Slider {
    pub min: f32,
    pub max: f32,
    pub label: &'static str,
}
impl InspectableRenderer<f32> for Slider {
    fn render_value<Value: InspectableValue<Value = f32>>(self, ui: &mut egui::Ui, val: &Value) {
        let mut num = val.get_value();
        ui.add(egui::Slider::new(&mut num, self.min..=self.max).text(self.label));
        val.set_value(num);
    }
}

pub struct InfoBox {
    pub label: &'static str,
}
impl<T: ToString> InspectableRendererRead<T> for InfoBox {
    fn render_value<Value: InspectableValueRead<Value = T>>(self, ui: &mut egui::Ui, val: &Value) {
        let mut text = val.get_value().to_string();
        ui.horizontal(|ui| {
            ui.label(self.label);
            ui.add_enabled(false, TextEdit::singleline(&mut text));
        });
    }
}
