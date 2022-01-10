use std::{cell::Cell, rc::Weak};

use egui::TextEdit;
use gfx_maths::Vec4;

use crate::scene::{
    entity::Entity,
    light::{DirectionalLight, Light},
};

use super::Component;

#[derive(Debug)]
pub struct LightComponent {
    entity: Weak<Entity>,
    pub light: Cell<Light>,
}

impl Component for LightComponent {
    fn create(entity: &std::rc::Rc<crate::scene::entity::Entity>) -> std::rc::Rc<Self>
    where
        Self: Sized,
    {
        std::rc::Rc::new(Self {
            entity: std::rc::Rc::downgrade(entity),
            light: Cell::new(Light::Directional(DirectionalLight {
                direction: Vec4::new(0., 1., 0., 0.0),
                illuminance: Vec4::new(10.1, 10.1, 10.1, 0.0),
            })),
        })
    }

    fn inspector_name(&self) -> &'static str {
        "LightComponent"
    }

    fn load(&self) {}

    fn start(&self) {}

    fn update(&self, _input: &crate::core::input::Input, _delta: f32) {
        if let Some(entity) = self.entity.upgrade() {
            match self.light.get() {
                Light::Directional(mut dl) => {
                    let new_dir = entity.get_global_rotation().forward();
                    dl.direction = Vec4::new(new_dir.x, new_dir.y, new_dir.z, 0.0);
                    self.light.set(Light::Directional(dl));
                }
                Light::Point(mut pl) => {
                    let new_pos = entity.get_global_position();
                    pl.position = Vec4::new(new_pos.x, new_pos.y, new_pos.z, 0.0);
                    self.light.set(Light::Point(pl));
                }
            }
        }
    }

    fn render(
        &self,
        _models: &mut Vec<(
            crate::scene::transform::TransformData,
            std::rc::Rc<crate::scene::model::Model>,
        )>,
        lights: &mut Vec<Light>,
    ) {
        lights.push(self.light.get());
    }

    fn render_inspector(&self, ui: &mut egui::Ui) {
        let light = self.light.get();

        if let Light::Directional(dl) = light {
            ui.label("Direction");

            let mut text_x = dl.direction.x.to_string();
            let mut text_y = dl.direction.y.to_string();
            let mut text_z = dl.direction.z.to_string();

            ui.add_enabled(false, TextEdit::singleline(&mut text_x));
            ui.add_enabled(false, TextEdit::singleline(&mut text_y));
            ui.add_enabled(false, TextEdit::singleline(&mut text_z));
        }

        self.light.set(light);
    }
}
