use std::{
    cell::Cell,
    rc::{Rc, Weak},
};

use egui::{Slider, TextEdit};
use gfx_maths::{Quaternion, Vec3};
use winit::keyboard::KeyCode;

use crate::scene::entity::Entity;

use super::Component;

#[derive(Debug)]
pub struct DebugMovementComponent {
    entity: Weak<Entity>,
    movement_speed: Cell<f32>,
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
            movement_speed: 5.0.into(),
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
        if input.get_button_down(KeyCode::KeyW) {
            movement += Vec3::new(0.0, 0.0, 1.0);
        }
        if input.get_button_down(KeyCode::KeyA) {
            movement += Vec3::new(-1.0, 0.0, 0.0);
        }
        if input.get_button_down(KeyCode::KeyS) {
            movement += Vec3::new(0.0, 0.0, -1.0);
        }
        if input.get_button_down(KeyCode::KeyD) {
            movement += Vec3::new(1.0, 0.0, 0.0);
        }
        if input.get_button_down(KeyCode::Space) {
            movement += Vec3::new(0.0, 1.0, 0.0);
        }
        if input.get_button_down(KeyCode::ControlLeft) {
            movement += Vec3::new(0.0, -1.0, 0.0);
        }
        if movement.sqr_magnitude() > 0.0 {
            movement.normalize();
        }

        let entity = self.entity.upgrade().unwrap();
        let mut transform = entity.transform.borrow_mut();

        let mut position = transform.position;
        position += rotation * movement * self.movement_speed.get() * delta;
        transform.position = position;

        transform.rotation = rotation;
    }

    fn inspector_name(&self) -> &'static str {
        "DebugMovementComponent"
    }

    fn render_inspector(&self, ui: &mut egui::Ui) {
        let mut ms = self.movement_speed.get();
        ui.add(Slider::new(&mut ms, 0.1..=20.0).text("Movement Speed"));
        self.movement_speed.set(ms);

        ui.horizontal(|ui| {
            ui.label("Rotation X");
            let mut text = self.rotation_x.get().to_degrees().to_string();
            ui.add_enabled(false, TextEdit::singleline(&mut text));
        });

        ui.horizontal(|ui| {
            ui.label("Rotation Y");
            let mut text = self.rotation_y.get().to_degrees().to_string();
            ui.add_enabled(false, TextEdit::singleline(&mut text));
        });
    }
}
