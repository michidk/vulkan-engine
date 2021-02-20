use math::prelude::*;

use super::buffer;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Camera {
    view_matrix: Mat4<f32>,
    position: Vec3<f32>,
    view_direction: Unit<Vec3<f32>>,
    down_direction: Unit<Vec3<f32>>,
    fovy: f32,
    aspect: f32,
    near: f32,
    far: f32,
    projection_matrix: Mat4<f32>,
}

impl Camera {
    pub fn update_buffer(&self, allocator: &vk_mem::Allocator, buffer: &mut buffer::BufferWrapper) {
        let data: [[[f32; 4]; 4]; 2] = [self.view_matrix.into(), self.projection_matrix.into()];
        buffer.fill(allocator, &data).unwrap();
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

    fn update_view_matrix(&mut self) {
        // TODO: Unit
        let right = Unit::new_normalize(self.down_direction.cross_product(&self.view_direction));
        let m: Mat4<f32> = Mat4::new(
            *right.x(),
            *right.y(),
            *right.z(),
            -right.dot_product(&self.position),
            //
            *self.down_direction.x(),
            *self.down_direction.y(),
            *self.down_direction.z(),
            -self.down_direction.dot_product(&self.position),
            //
            *self.view_direction.x(),
            *self.view_direction.y(),
            *self.view_direction.z(),
            -self.view_direction.dot_product(&self.position),
            //
            0.0,
            0.0,
            0.0,
            1.0,
        );
        self.view_matrix = m;
    }

    pub fn move_forward(&mut self, distance: f32) {
        self.position += self.view_direction.as_ref() * distance;
        self.update_view_matrix();
    }

    pub fn move_backward(&mut self, distance: f32) {
        self.move_forward(-distance);
    }

    pub fn turn_right(&mut self, angle: Angle<f32>) {
        let rotation = Mat3::from_axis_angle(&self.down_direction, angle);
        self.view_direction = Unit::new_normalize(&rotation * self.view_direction.as_ref());
        self.update_view_matrix();
    }

    pub fn turn_left(&mut self, angle: Angle<f32>) {
        self.turn_right(-angle);
    }

    pub fn turn_up(&mut self, angle: Angle<f32>) {
        let right = Unit::new_normalize(self.down_direction.cross_product(&self.view_direction));
        let rotation = Mat3::from_axis_angle(&right, angle);
        self.view_direction = Unit::new_normalize(&rotation * self.view_direction.as_ref());
        self.down_direction = Unit::new_normalize(&rotation * self.down_direction.as_ref());
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
            view_direction: Unit::new_normalize(Vec3::new(0.0, 1.0, 1.0)),
            down_direction: Unit::new_normalize(Vec3::new(0.0, 1.0, -1.0)),
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
    view_direction: Unit<Vec3<f32>>,
    down_direction: Unit<Vec3<f32>>,
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

    pub fn view_direction(&mut self, direction: Vec3<f32>) -> &mut Self {
        self.view_direction = Unit::new_normalize(direction);
        self
    }

    pub fn down_direction(&mut self, direction: Vec3<f32>) -> &mut Self {
        self.down_direction = Unit::new_normalize(direction);
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
        let down = self.down_direction.as_ref();
        let view = self.view_direction.as_ref();

        let dv = view * down.dot_product(view);
        let ds = down - &dv;

        let mut cam = Camera {
            position: self.position,
            view_direction: self.view_direction,
            down_direction: Unit::new_normalize(ds),
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
