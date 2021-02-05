use std::ops::{Add, AddAssign, Mul, MulAssign};

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub const fn fill(value: f32) -> Self {
        Self {
            x: value,
            y: value,
            z: value,
        }
    }

    pub const fn to_vec4(self, w: f32) -> Vec4 {
        Vec4::new(self.x, self.y, self.z, w)
    }
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub const fn fill(value: f32) -> Self {
        Self {
            x: value,
            y: value,
            z: value,
            w: value,
        }
    }

    pub fn dot_product(self, other: Self) -> f32 {
        (self.x * other.x)
            + (self.y * other.y)
            + (self.z * other.z)
            + (self.w * other.w)
    }
}

impl From<Vec4> for [f32; 4] {
    fn from(value: Vec4) -> Self {
        [value.x, value.y, value.z, value.w]
    }
}

impl Add<Vec4> for Vec4 {
    type Output = Vec4;

    fn add(mut self, rhs: Vec4) -> Self::Output {
        self += rhs;
        self
    }
}

impl AddAssign<Vec4> for Vec4 {
    fn add_assign(&mut self, rhs: Vec4) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
        self.w += rhs.w;
    }
}

impl Mul<f32> for Vec4 {
    type Output = Self;

    fn mul(mut self, rhs: f32) -> Self::Output {
        self *= rhs;
        self
    }
}

impl MulAssign<f32> for Vec4 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
        self.w *= rhs;
    }
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Mat4 {
    pub x: Vec4,
    pub y: Vec4,
    pub z: Vec4,
    pub w: Vec4,
}

impl Mat4 {
    pub const ZERO: Mat4 = Self::new(
        Vec4::ZERO,
        Vec4::ZERO,
        Vec4::ZERO,
        Vec4::ZERO,
    );

    pub const IDENTITY: Mat4 = Self::new(
        Vec4::new(1.0, 0.0, 0.0, 0.0),
        Vec4::new(0.0, 1.0, 0.0, 0.0),
        Vec4::new(0.0, 0.0, 1.0, 0.0),
        Vec4::new(0.0, 0.0, 0.0, 1.0),
    );

    pub const fn new(x: Vec4, y: Vec4, z: Vec4, w: Vec4) -> Self {
        Self { x, y, z, w }
    }

    pub fn scaling(factor: f32) -> Self {
        let mut scaling = Self::IDENTITY * factor;
        // set [3, 3] to one
        scaling.w.w = 1.0;
        scaling
    }

    pub fn translation(direction: Vec3) -> Self {
        let mut translating = Self::IDENTITY;
        translating.w.x = direction.x;
        translating.w.y = direction.y;
        translating.w.z = direction.z;
        translating
    }

    fn col1(&self) -> Vec4 {
        Vec4::new(self.x.x, self.y.x, self.z.x, self.w.x)
    }

    fn col2(&self) -> Vec4 {
        Vec4::new(self.x.y, self.y.y, self.z.y, self.w.y)
    }

    fn col3(&self) -> Vec4 {
        Vec4::new(self.x.z, self.y.z, self.z.z, self.w.z)
    }

    fn col4(&self) -> Vec4 {
        Vec4::new(self.x.w, self.y.w, self.z.w, self.w.w)
    }
}

impl From<Mat4> for [[f32; 4]; 4] {
    fn from(value: Mat4) -> Self {
        [
            value.x.into(),
            value.y.into(),
            value.z.into(),
            value.w.into(),
        ]
    }
}

impl Add<Mat4> for Mat4 {
    type Output = Mat4;

    fn add(mut self, rhs: Mat4) -> Self::Output {
        self += rhs;
        self
    }
}

impl AddAssign<Mat4> for Mat4 {
    fn add_assign(&mut self, rhs: Mat4) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
        self.w += rhs.w;
    }
}

impl Mul<f32> for Mat4 {
    type Output = Self;

    fn mul(mut self, rhs: f32) -> Self::Output {
        self *= rhs;
        self
    }
}

impl MulAssign<f32> for Mat4 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
        self.w *= rhs;
    }
}

impl Mul<Mat4> for Mat4 {
    type Output = Self;

    fn mul(mut self, rhs: Mat4) -> Self::Output {
        self *= rhs;
        self
    }
}

impl MulAssign<Mat4> for Mat4 {
    fn mul_assign(&mut self, rhs: Mat4) {
        let mut tmp = Mat4::ZERO;

        tmp.x.x = self.x.dot_product(rhs.col1());
        tmp.x.y = self.x.dot_product(rhs.col2());
        tmp.x.z = self.x.dot_product(rhs.col3());
        tmp.x.w = self.x.dot_product(rhs.col4());

        tmp.y.x = self.y.dot_product(rhs.col1());
        tmp.y.y = self.y.dot_product(rhs.col2());
        tmp.y.z = self.y.dot_product(rhs.col3());
        tmp.y.w = self.y.dot_product(rhs.col4());

        tmp.z.x = self.z.dot_product(rhs.col1());
        tmp.z.y = self.z.dot_product(rhs.col2());
        tmp.z.z = self.z.dot_product(rhs.col3());
        tmp.z.w = self.z.dot_product(rhs.col4());

        tmp.w.x = self.w.dot_product(rhs.col1());
        tmp.w.y = self.w.dot_product(rhs.col2());
        tmp.w.z = self.w.dot_product(rhs.col3());
        tmp.w.w = self.w.dot_product(rhs.col4());

        *self = tmp;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec4_mul() {
        let v = Vec4::new(0.1, 0.2, 1.0, -0.5);

        assert_eq!(v * 2.1, Vec4::new(0.21, 0.42, 2.1, -1.05));
    }

    #[test]
    fn mat4_scaling() {
        let m = Mat4::scaling(2.1);

        assert_eq!(
            m,
            Mat4::new(
                Vec4::new(2.1, 0.0, 0.0, 0.0),
                Vec4::new(0.0, 2.1, 0.0, 0.0),
                Vec4::new(0.0, 0.0, 2.1, 0.0),
                Vec4::new(0.0, 0.0, 0.0, 2.1),
            )
        )
    }
}
