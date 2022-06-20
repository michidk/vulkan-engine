use std::{
    cell::Cell,
    rc::{Rc, Weak},
};

use egui::Slider;
use gfx_maths::{Quaternion, Vec3};

use crate::{core::input::Input, scene::entity::Entity};

use super::Component;

#[derive(Debug)]
pub struct RotationComponent {
    entity: Weak<Entity>,
    pub rotation_speed: Cell<f32>,
    pub axis: Cell<Vec3>,
}

impl Component for RotationComponent {
    fn create(entity: &Rc<Entity>) -> Rc<Self>
    where
        Self: Sized,
    {
        Rc::new(Self {
            entity: Rc::downgrade(entity),
            rotation_speed: Cell::new(30.0),
            axis: Cell::new(Vec3::new(0.0, 0.0, 1.0)),
        })
    }

    fn load(&self) {}

    fn start(&self) {}

    fn update(&self, _: &Input, delta: f32) {
        if let Some(entity) = self.entity.upgrade() {
            let mut transform = entity.transform.borrow_mut();

            let mut rotation = transform.rotation;
            rotation = Quaternion::axis_angle(
                self.axis.get(),
                self.rotation_speed.get().to_radians() * delta,
            ) * rotation;
            transform.rotation = rotation;
        }
    }

    fn inspector_name(&self) -> &'static str {
        "RotationComponent"
    }

    fn render_inspector(&self, ui: &mut egui::Ui) {
        let mut rot_speed = self.rotation_speed.get();
        ui.add(Slider::new(&mut rot_speed, -360.0..=360.0).text("Rotation Speed"));
        self.rotation_speed.set(rot_speed);
    }
}
