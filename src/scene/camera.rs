use crystal::prelude::*;

use crate::vulkan::buffer::{self, MutableBuffer, PerFrameUniformBuffer};

pub struct CamData {
    pub view_matrix: [[f32; 4]; 4],
    pub projection_matrix: [[f32; 4]; 4]
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Camera {
    view_matrix: Mat4<f32>,
    position: Vec3<f32>,
    rotation_x: f32,
    rotation_y: f32,
    fovy: f32,
    aspect: f32,
    near: f32,
    far: f32,
    projection_matrix: Mat4<f32>,
}

impl Camera {
    pub fn update_buffer(&self, allocator: &vk_mem::Allocator, buffer: &mut buffer::PerFrameUniformBuffer<CamData>) {
        let cam_data = CamData {
            view_matrix: self.view_matrix.into(),
            projection_matrix: self.projection_matrix.into()
        };
        buffer.set_data(allocator, &cam_data);
    }

    fn update_projection_matrix(&mut self) {
        let d = 1.0 / (0.5 * self.fovy).tan();
        self.projection_matrix = Mat4::new(
            d / self.aspect,
            0.0,
            0.0,
            0.0,
            0.0,
            d,
            0.0,
            0.0,
            0.0,
            0.0,
            self.far / (self.far - self.near),
            -self.near * self.far / (self.far - self.near),
            0.0,
            0.0,
            1.0,
            0.0,
        );
    }

    fn get_rotation(&self) -> Quaternion<f32> {
        Quaternion::from_axis_angle(
            Unit::new_normalize(Vec3::new(0.0, 1.0, 0.0)),
            Angle::from_rad(self.rotation_y),
        ) * Quaternion::from_axis_angle(
            Unit::new_normalize(Vec3::new(1.0, 0.0, 0.0)),
            Angle::from_rad(self.rotation_x),
        )
    }

    fn update_view_matrix(&mut self) {
        let rotation = self.get_rotation();

        let forward_dir = rotation.forward();
        let right_dir = rotation.right();
        let up_dir = rotation.up();

        let m: Mat4<f32> = Mat4::new(
            *right_dir.x(),
            *right_dir.y(),
            *right_dir.z(),
            -right_dir.dot_product(&self.position),
            //
            *up_dir.x(),
            *up_dir.y(),
            *up_dir.z(),
            -up_dir.dot_product(&self.position),
            //
            *forward_dir.x(),
            *forward_dir.y(),
            *forward_dir.z(),
            -forward_dir.dot_product(&self.position),
            //
            0.0,
            0.0,
            0.0,
            1.0,
        );
        self.view_matrix = m;
    }

    pub fn move_in_view_direction(&mut self, movement: &Vec3<f32>) {
        let rotation = self.get_rotation();
        self.position += &rotation * movement;
        self.update_view_matrix();
    }

    pub fn rotate(&mut self, angle_x: Angle<f32>, angle_y: Angle<f32>) {
        self.rotation_y += angle_y.to_rad();
        self.rotation_x = (self.rotation_x + angle_x.to_rad())
            .min(Angle::from_deg(85.0).to_rad())
            .max(Angle::from_deg(-85.0).to_rad());
        self.update_view_matrix();
    }

    pub fn turn_up(&mut self, angle: Angle<f32>) {
        self.rotation_x += angle.to_rad();
        if self.rotation_x < Angle::from_deg(-85.0).to_rad() {
            self.rotation_x = Angle::from_deg(-85.0).to_rad();
        }
        if self.rotation_x > Angle::from_deg(85.0).to_rad() {
            self.rotation_x = Angle::from_deg(85.0).to_rad();
        }

        self.update_view_matrix();
    }

    pub fn turn_down(&mut self, angle: Angle<f32>) {
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
}

#[allow(dead_code)]
pub struct CameraBuilder {
    position: Vec3<f32>,
    rotation: (f32, f32),
    fovy: f32,
    aspect: f32,
    near: f32,
    far: f32,
}

impl CameraBuilder {
    pub fn position(&mut self, pos: Vec3<f32>) -> &mut Self {
        self.position = pos;
        self
    }

    pub fn rotation(&mut self, rotation: (f32, f32)) -> &mut Self {
        self.rotation = rotation;
        self
    }

    pub fn fovy(&mut self, fovy: Angle<f32>) -> &mut Self {
        let fovy = fovy.to_rad();
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
        };
        cam.update_projection_matrix();
        cam.update_view_matrix();
        cam
    }
}
