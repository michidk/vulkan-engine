use std::ops::Mul;

use crate::angle::Angle;
use crate::unit::Unit;
use crate::vector::Vec3;
use crate::{norm::Normed, scalar::Scalar};

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quaternion {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Quaternion {
        Quaternion { x, y, z, w }
    }

    pub fn from_axis_angle(axis: &Unit<Vec3<f32>>, angle: Angle<f32>) -> Quaternion {
        let mut axis = *axis.as_ref();
        axis.scale_mut((angle.to_rad() * 0.5f32).sin());

        Quaternion {
            x: *axis.x(),
            y: *axis.y(),
            z: *axis.z(),
            w: (angle.to_rad() * 0.5f32).cos(),
        }
    }

    pub fn conjugated(&self) -> Quaternion {
        Quaternion {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: self.w,
        }
    }

    pub fn forward(&self) -> Vec3<f32> {
        *self * &Vec3::new(0.0, 0.0, 1.0)
    }
    pub fn right(&self) -> Vec3<f32> {
        *self * &Vec3::new(1.0, 0.0, 0.0)
    }
    pub fn up(&self) -> Vec3<f32> {
        *self * &Vec3::new(0.0, 1.0, 0.0)
    }
}

impl Normed for Quaternion {
    type Norm = f32;

    fn norm(&self) -> Self::Norm {
        self.norm_squared().sqrt()
    }

    fn norm_squared(&self) -> Self::Norm {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }

    fn scale_mut(&mut self, n: Self::Norm) {
        self.x *= n;
        self.y *= n;
        self.z *= n;
        self.w *= n;
    }

    fn unscale_mut(&mut self, n: Self::Norm) {
        self.x /= n;
        self.y /= n;
        self.z /= n;
        self.w /= n;
    }
}

impl Mul<Quaternion> for Quaternion {
    type Output = Quaternion;

    fn mul(self, rhs: Quaternion) -> Quaternion {
        Quaternion {
            x: self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            y: self.w * rhs.y + self.y * rhs.w + self.z * rhs.x - self.x * rhs.z,
            z: self.w * rhs.z + self.z * rhs.w + self.x * rhs.y - self.y * rhs.x,
            w: self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
        }
    }
}

impl Mul<&Vec3<f32>> for Quaternion {
    type Output = Vec3<f32>;

    fn mul(self, rhs: &Vec3<f32>) -> Vec3<f32> {
        let res = self * Quaternion::new(*rhs.x(), *rhs.y(), *rhs.z(), 0.0) * self.conjugated();
        Vec3::new(res.x, res.y, res.z)
    }
}
