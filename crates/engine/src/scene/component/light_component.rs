use std::{cell::Cell, rc::Weak};

use egui::DragValue;
use gfx_maths::{Vec3, Vec4};

use crate::scene::{
    entity::Entity,
    light::{DirectionalLight, Light},
};

use super::Component;

#[derive(Debug)]
pub struct LightComponent {
    entity: Weak<Entity>,
    pub light: Cell<Light>,

    color: Cell<Vec3>,
    intensity: Cell<f32>,
}

impl Component for LightComponent {
    fn create(entity: &std::rc::Rc<crate::scene::entity::Entity>) -> std::rc::Rc<Self>
    where
        Self: Sized,
    {
        let color = Vec3::one();
        let intensity = 10.0;

        std::rc::Rc::new(Self {
            entity: std::rc::Rc::downgrade(entity),
            light: Cell::new(Light::Directional(DirectionalLight {
                direction: Vec4::new(0., 1., 0., 0.0),
                illuminance: Vec4::new(color.x, color.y, color.z, 0.0) * intensity,
            })),

            color: color.into(),
            intensity: intensity.into(),
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
        let mut light = self.light.get();

        let col = self.color.get();
        let mut col = [col.x, col.y, col.z];
        let mut int = self.intensity.get();

        ui.label("Color");
        if ui.color_edit_button_rgb(&mut col).changed()
            || ui
                .add(DragValue::new(&mut int).prefix("Intensity: "))
                .changed()
        {
            self.color.set(Vec3::new(col[0], col[1], col[2]));
            self.intensity.set(int);

            match &mut light {
                Light::Directional(dl) => {
                    dl.illuminance = Vec4::new(col[0], col[1], col[2], 0.0) * int
                }
                Light::Point(pl) => pl.luminous_flux = Vec4::new(col[0], col[1], col[2], 0.0) * int,
            }
        }

        self.light.set(light);
    }
}
