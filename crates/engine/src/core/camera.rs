use std::time::Instant;

use gfx_maths::*;
use winit::event::VirtualKeyCode;

use crate::vulkan::{
    allocator::Allocator,
    buffer::{self, MutableBuffer},
};

use super::input::Input;

pub struct CamData {
    pub view_matrix: Mat4,
    pub projection_matrix: Mat4,
    pub inv_view_matrix: Mat4,
    pub inv_projection_matrix: Mat4,
    pub pos: Vec3,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Camera {
    view_matrix: Mat4,
    position: Vec3,
    rotation_x: f32,
    rotation_y: f32,
    fovy: f32,
    aspect: f32,
    near: f32,
    far: f32,
    projection_matrix: Mat4,
    inv_view_matrix: Mat4,
    inv_projection_matrix: Mat4,
    last_render: Instant,
}

impl Camera {
    pub fn update_buffer(
        &self,
        allocator: &Allocator,
        buffer: &mut buffer::PerFrameUniformBuffer<CamData>,
        current_frame_index: u8,
    ) {
        let cam_data = CamData {
            view_matrix: self.view_matrix,
            projection_matrix: self.projection_matrix,
            inv_view_matrix: self.inv_view_matrix,
            inv_projection_matrix: self.inv_projection_matrix,
            pos: self.position,
        };
        buffer
            .set_data(allocator, &cam_data, current_frame_index)
            .unwrap();
    }

    fn update_projection_matrix(&mut self) {
        self.projection_matrix =
            Mat4::perspective_vulkan(self.fovy, self.near, self.far, self.aspect);
        self.inv_projection_matrix =
            Mat4::inverse_perspective_vulkan(self.fovy, self.near, self.far, self.aspect);
    }

    fn get_rotation(&self) -> Quaternion {
        Quaternion::axis_angle(Vec3::new(0.0, 1.0, 0.0), self.rotation_y)
            * Quaternion::axis_angle(Vec3::new(1.0, 0.0, 0.0), self.rotation_x)
    }

    fn update_view_matrix(&mut self) {
        let rotation = self.get_rotation();

        let m = Mat4::rotate(-rotation) * Mat4::translate(-self.position);
        let im = Mat4::translate(self.position) * Mat4::rotate(rotation);

        self.view_matrix = m;
        self.inv_view_matrix = im;
    }

    pub fn move_in_view_direction(&mut self, movement: &Vec3) {
        let rotation = self.get_rotation();
        self.position += rotation * movement;
        self.update_view_matrix();
    }

    pub fn rotate(&mut self, angle_x: f32, angle_y: f32) {
        self.rotation_y += angle_y;
        self.rotation_x = (self.rotation_x + angle_x)
            .min(85.0f32.to_radians())
            .max(-85.0f32.to_radians());
        self.update_view_matrix();
    }

    pub fn turn_up(&mut self, angle: f32) {
        self.rotation_x = (self.rotation_x + angle)
            .min(85.0f32.to_radians())
            .max(-85.0f32.to_radians());

        self.update_view_matrix();
    }

    pub fn turn_down(&mut self, angle: f32) {
        self.turn_up(-angle);
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
        self.update_projection_matrix();
    }

    pub fn builder() -> CameraBuilder {
        CameraBuilder {
            position: Vec3::new(0.0, -3.0, -3.0),
            rotation: (0.0, 0.0),
            fovy: std::f32::consts::FRAC_PI_3,
            aspect: 800.0 / 600.0,
            near: 0.1,
            far: 100.0,
        }
    }

    // TODO: implement seperate movement struct as soon as gameloop is
    pub(crate) fn movement(&mut self, input: &Input) {
        let delta = self.last_render.elapsed().as_secs_f32();
        self.last_render = Instant::now();

        let mouse_sensitivity = 0.123f32;

        self.rotate(
            (input.get_mouse_delta().1 as f32 * mouse_sensitivity).to_radians(),
            (input.get_mouse_delta().0 as f32 * mouse_sensitivity).to_radians(),
        );

        let mut movement: Vec3 = Vec3::zero();

        // WASD movement
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
        // UP, DOWN
        if input.get_button_down(VirtualKeyCode::Space) {
            movement += Vec3::new(0.0, 1.0, 0.0);
        }
        if input.get_button_down(VirtualKeyCode::LControl) {
            movement += Vec3::new(0.0, -1.0, 0.0);
        }

        if movement.sqr_magnitude() > 0.0 {
            movement.normalize();
        }

        let move_speed: f32 = 5.0;
        let move_step = movement * (move_speed * delta);
        self.move_in_view_direction(&move_step);
    }
}

pub struct CameraBuilder {
    position: Vec3,
    rotation: (f32, f32),
    fovy: f32,
    aspect: f32,
    near: f32,
    far: f32,
}

impl CameraBuilder {
    pub fn position(&mut self, pos: Vec3) -> &mut Self {
        self.position = pos;
        self
    }

    pub fn rotation(&mut self, rotation: (f32, f32)) -> &mut Self {
        self.rotation = rotation;
        self
    }

    pub fn fovy(&mut self, fovy: f32) -> &mut Self {
        let fovy = fovy.to_radians();
        const MIN: f32 = 0.01;
        const MAX: f32 = std::f32::consts::PI - 0.01;

        self.fovy = fovy.max(MIN).min(MAX);
        if (self.fovy - fovy).abs() > 1e-6 {
            log::warn!("FovY out of bounds: {} <= `{}` <= {}", MIN, fovy, MAX);
        }
        self
    }

    pub fn aspect(&mut self, aspect: f32) -> &mut Self {
        self.aspect = aspect;
        self
    }

    pub fn near(&mut self, near: f32) -> &mut Self {
        if near <= 0.0 {
            log::warn!("Near is negative: `{}`", near);
        }
        self.near = near;
        self
    }

    pub fn far(&mut self, far: f32) -> &mut Self {
        if far <= 0.0 {
            log::warn!("Far is negative: `{}`", far);
        }
        self.far = far;
        self
    }

    pub fn build(&mut self) -> Camera {
        if self.far < self.near {
            log::warn!("Far is closer than near: `{}` `{}`", self.far, self.near);
        }

        let mut cam = Camera {
            position: self.position,
            rotation_x: self.rotation.0,
            rotation_y: self.rotation.1,
            fovy: self.fovy,
            aspect: self.aspect,
            near: self.near,
            far: self.far,
            view_matrix: Mat4::identity(),
            projection_matrix: Mat4::identity(),
            inv_view_matrix: Mat4::identity(),
            inv_projection_matrix: Mat4::identity(),
            last_render: Instant::now(),
        };
        cam.update_projection_matrix();
        cam.update_view_matrix();
        cam
    }
}
