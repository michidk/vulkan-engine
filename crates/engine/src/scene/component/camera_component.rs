use std::{
    cell::Cell,
    rc::{Rc, Weak},
};

use gfx_maths::{Mat4, Quaternion, Vec3};
use winit::event::VirtualKeyCode;

use crate::{core::input::Input, scene::entity::Entity};

use super::Component;

#[derive(Debug)]
pub struct CameraComponent {
    entity: Weak<Entity>,
    near: f32,
    far: f32,
    fovy: f32,
    rotation_x: Cell<f32>,
    rotation_y: Cell<f32>,
}

impl Component for CameraComponent {
    fn create(entity: &Rc<Entity>) -> Rc<Self>
    where
        Self: Sized,
    {
        let res = Rc::new(Self {
            entity: Rc::downgrade(entity),
            near: 0.01,
            far: 1000.0,
            fovy: 60.0,
            rotation_x: 0.0.into(),
            rotation_y: 0.0.into(),
        });

        if let Some(scene) = entity.scene.upgrade() {
            scene.set_main_camera(Rc::downgrade(&res));
        }

        res
    }

    fn load(&self) {}

    fn start(&self) {}

    fn update(&self, input: &Input, delta: f32) {
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
        position += rotation * movement * 5.0 * delta;
        transform.position = position;

        transform.rotation = rotation;
    }
}

#[repr(C)]
pub(crate) struct CamData {
    pub view_matrix: Mat4,
    pub projection_matrix: Mat4,
    pub inv_view_matrix: Mat4,
    pub inv_projection_matrix: Mat4,
    pub pos: Vec3,
}

impl CameraComponent {
    pub(crate) fn get_cam_data(&self, aspect: f32) -> CamData {
        let entity = self.entity.upgrade().unwrap();
        let transform = entity.transform.borrow();

        CamData {
            view_matrix: Mat4::rotate(-transform.rotation) * Mat4::translate(-transform.position),
            projection_matrix: Mat4::perspective_vulkan(
                self.fovy.to_radians(),
                self.near,
                self.far,
                aspect,
            ),
            inv_view_matrix: Mat4::translate(transform.position) * Mat4::rotate(transform.rotation),
            inv_projection_matrix: Mat4::inverse_perspective_vulkan(
                self.fovy.to_radians(),
                self.near,
                self.far,
                aspect,
            ),
            pos: transform.position,
        }
    }
}
