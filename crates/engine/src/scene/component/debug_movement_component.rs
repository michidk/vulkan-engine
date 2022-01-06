use std::{
    cell::Cell,
    rc::{Rc, Weak},
};

use gfx_maths::{Quaternion, Vec3};
use winit::event::VirtualKeyCode;

use crate::scene::entity::Entity;

use super::Component;

#[derive(Debug)]
pub struct DebugMovementComponent {
    entity: Weak<Entity>,
    movement_speed: f32,
    rotation_x: Cell<f32>,
    rotation_y: Cell<f32>,
}

impl Component for DebugMovementComponent {
    fn create(entity: &std::rc::Rc<Entity>) -> std::rc::Rc<Self>
    where
        Self: Sized,
    {
        Rc::new(Self {
            entity: Rc::downgrade(entity),
            movement_speed: 5.0,
            rotation_x: 0.0.into(),
            rotation_y: 0.0.into(),
        })
    }

    fn load(&self) {}

    fn start(&self) {}

    fn update(&self, input: &crate::core::input::Input, delta: f32) {
        if !input.get_cursor_captured() {
            return;
        }

        let mouse_sensitivity = 0.123f32;

        let mut rot_x = self.rotation_x.get();
        let mut rot_y = self.rotation_y.get();
        rot_y += (input.get_mouse_delta().0 as f32 * mouse_sensitivity).to_radians();
        rot_x = (rot_x + (input.get_mouse_delta().1 as f32 * mouse_sensitivity).to_radians())
            .min(85.0f32.to_radians())
            .max(-85.0f32.to_radians());
        self.rotation_y.set(rot_y);
        self.rotation_x.set(rot_x);

        let rotation = Quaternion::axis_angle(Vec3::new(0.0, 1.0, 0.0), rot_y)
            * Quaternion::axis_angle(Vec3::new(1.0, 0.0, 0.0), rot_x);

        let mut movement = Vec3::zero();
        if input.get_button_down(VirtualKeyCode::W) {
            movement += Vec3::new(0.0, 0.0, 1.0);
        }
        if input.get_button_down(VirtualKeyCode::A) {
            movement += Vec3::new(-1.0, 0.0, 0.0);
        }
        if input.get_button_down(VirtualKeyCode::S) {
            movement += Vec3::new(0.0, 0.0, -1.0);
        }
        if input.get_button_down(VirtualKeyCode::D) {
            movement += Vec3::new(1.0, 0.0, 0.0);
        }
        if input.get_button_down(VirtualKeyCode::Space) {
            movement += Vec3::new(0.0, 1.0, 0.0);
        }
        if input.get_button_down(VirtualKeyCode::LControl) {
            movement += Vec3::new(0.0, -1.0, 0.0);
        }
        if movement.sqr_magnitude() > 0.0 {
            movement.normalize();
        }

        let entity = self.entity.upgrade().unwrap();
        let mut transform = entity.transform.borrow_mut();

        let mut position = transform.position;
        position += rotation * movement * self.movement_speed * delta;
        transform.position = position;

        transform.rotation = rotation;
    }

    fn inspector_name(&self) -> &'static str {
        "DebugMovementComponent"
    }
}
