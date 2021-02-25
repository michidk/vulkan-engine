use std::fmt;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, Sub};

use crate::angle::{Angle, AngleConst};
use crate::scalar::{Cos, One, Sin, Sqrt, Zero};
use crate::unit::Unit;
use crate::vector::{Vec3, Vec4};
use crate::{norm::Normed, scalar::Scalar};

pub struct Quaternion<T> {
    inner: Vec4<T>,
}

impl<T> Quaternion<T> {
    pub fn new(x: T, y: T, z: T, w: T) -> Self {
        Self {
            inner: Vec4::new(x, y, z, w),
        }
    }

    pub fn from_vec4(vec4: Vec4<T>) -> Self {
        Self { inner: vec4 }
    }

    pub fn x(&self) -> &T {
        self.inner.x()
    }

    pub fn y(&self) -> &T {
        self.inner.y()
    }

    pub fn z(&self) -> &T {
        self.inner.z()
    }

    pub fn w(&self) -> &T {
        self.inner.w()
    }

    pub fn x_mut(&mut self) -> &mut T {
        self.inner.x_mut()
    }

    pub fn y_mut(&mut self) -> &mut T {
        self.inner.y_mut()
    }

    pub fn z_mut(&mut self) -> &mut T {
        self.inner.z_mut()
    }

    pub fn w_mut(&mut self) -> &mut T {
        self.inner.w_mut()
    }

    pub fn from_axis_angle(axis: Unit<Vec3<T>>, angle: Angle<T>) -> Self
    where
        T: Clone
            + AngleConst
            + Zero
            + One
            + PartialEq
            + Add<T, Output = T>
            + Mul<T, Output = T>
            + Div<T, Output = T>
            + Rem<T, Output = T>
            + Sin<Output = T>
            + Cos<Output = T>,
        Vec3<T>: Normed<Norm = T>,
    {
        let two = T::one() + T::one();
        let mut scaled = axis.into_inner();
        scaled.scale_mut((angle.to_rad() / two.clone()).sin());

        Self::new(
            scaled.x().clone(),
            scaled.y().clone(),
            scaled.z().clone(),
            (angle.to_rad() / two).cos(),
        )
    }

    pub fn conjugated(&self) -> Self
    where
        T: Clone + Neg<Output = T>,
    {
        Self::new(
            -self.x().clone(),
            -self.y().clone(),
            -self.z().clone(),
            self.inner.w().clone(),
        )
    }

    pub fn forward(&self) -> Vec3<T>
    where
        T: Clone
            + Zero
            + One
            + Add<T, Output = T>
            + Sub<T, Output = T>
            + Mul<T, Output = T>
            + Neg<Output = T>,
    {
        self * &Vec3::unit_z()
    }

    pub fn right(&self) -> Vec3<T>
    where
        T: Clone
            + Zero
            + One
            + Add<T, Output = T>
            + Sub<T, Output = T>
            + Mul<T, Output = T>
            + Neg<Output = T>,
    {
        self * &Vec3::unit_x()
    }

    pub fn up(&self) -> Vec3<T>
    where
        T: Clone
            + Zero
            + One
            + Add<T, Output = T>
            + Sub<T, Output = T>
            + Mul<T, Output = T>
            + Neg<Output = T>,
    {
        self * &Vec3::unit_y()
    }
}

impl<T> Clone for Quaternion<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self::from_vec4(self.inner.clone())
    }
}

impl<T> Copy for Quaternion<T> where T: Copy {}

impl<T> fmt::Debug for Quaternion<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Quaternion").field(&self.inner).finish()
    }
}

impl<T> Default for Quaternion<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::from_vec4(Vec4::default())
    }
}

impl<T> PartialEq<Self> for Quaternion<T>
where
    T: PartialEq<T>,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl<T> From<[T; 4]> for Quaternion<T> {
    fn from(data: [T; 4]) -> Self {
        Self::from_vec4(data.into())
    }
}

impl<'a, T> From<&'a Quaternion<T>> for Vec3<T>
where
    T: Clone,
{
    fn from(value: &'a Quaternion<T>) -> Self {
        Vec3::new(value.x().clone(), value.y().clone(), value.z().clone())
    }
}

impl<T> From<Quaternion<T>> for Vec3<T>
where
    T: Clone,
{
    fn from(value: Quaternion<T>) -> Self {
        Vec3::new(value.x().clone(), value.y().clone(), value.z().clone())
    }
}

impl<'a, 'b, T, RT> Mul<&'a Quaternion<RT>> for &'b Quaternion<T>
where
    T: Clone + Add<T, Output = T> + Sub<T, Output = T> + Mul<RT, Output = T>,
    RT: Clone,
{
    type Output = Quaternion<T>;

    #[rustfmt::skip]
    fn mul(self, rhs: &'a Quaternion<RT>) -> Self::Output {
        Quaternion::new(
            self.w().clone() * rhs.x().clone() + self.x().clone() * rhs.w().clone() + self.y().clone() * rhs.z().clone() - self.z().clone() * rhs.y().clone(),
            self.w().clone() * rhs.y().clone() + self.y().clone() * rhs.w().clone() + self.z().clone() * rhs.x().clone() - self.x().clone() * rhs.z().clone(),
            self.w().clone() * rhs.z().clone() + self.z().clone() * rhs.w().clone() + self.x().clone() * rhs.y().clone() - self.y().clone() * rhs.x().clone(),
            self.w().clone() * rhs.w().clone() - self.x().clone() * rhs.x().clone() - self.y().clone() * rhs.y().clone() - self.z().clone() * rhs.z().clone(),
        )
    }
}

impl<T, RT> Mul<Quaternion<RT>> for Quaternion<T>
where
    T: Clone + Add<T, Output = T> + Sub<T, Output = T> + Mul<RT, Output = T>,
    RT: Clone,
{
    type Output = Self;

    #[rustfmt::skip]
    fn mul(self, rhs: Quaternion<RT>) -> Self::Output {
        Self::new(
            self.w().clone() * rhs.x().clone() + self.x().clone() * rhs.w().clone() + self.y().clone() * rhs.z().clone() - self.z().clone() * rhs.y().clone(),
            self.w().clone() * rhs.y().clone() + self.y().clone() * rhs.w().clone() + self.z().clone() * rhs.x().clone() - self.x().clone() * rhs.z().clone(),
            self.w().clone() * rhs.z().clone() + self.z().clone() * rhs.w().clone() + self.x().clone() * rhs.y().clone() - self.y().clone() * rhs.x().clone(),
            self.w().clone() * rhs.w().clone() - self.x().clone() * rhs.x().clone() - self.y().clone() * rhs.y().clone() - self.z().clone() * rhs.z().clone(),
        )
    }
}

impl<'a, 'b, T, RT> Mul<&'a Vec3<RT>> for &'b Quaternion<T>
where
    T: Clone
        + Add<T, Output = T>
        + Sub<T, Output = T>
        + Mul<T, Output = T>
        + Mul<RT, Output = T>
        + Neg<Output = T>,
    RT: Clone + Zero,
{
    type Output = Vec3<T>;

    fn mul(self, rhs: &'a Vec3<RT>) -> Self::Output {
        let rhs = Quaternion::new(
            rhs.x().clone(),
            rhs.y().clone(),
            rhs.z().clone(),
            RT::zero(),
        );
        let res = &(self * &rhs) * &self.conjugated();
        res.into()
    }
}
//
//impl Mul<&Vec3<f32>> for Quaternion {
//    type Output = Vec3<f32>;
//
//    fn mul(self, rhs: &Vec3<f32>) -> Vec3<f32> {
//        let res = self * Quaternion::new(*rhs.x(), *rhs.y(), *rhs.z(), 0.0) * self.conjugated();
//        Vec3::new(res.x, res.y, res.z)
//    }
//}

impl<T> Normed for Quaternion<T>
where
    T: Clone
        + Sqrt<Output = T>
        + Add<T, Output = T>
        + Mul<T, Output = T>
        + MulAssign<T>
        + DivAssign<T>,
{
    type Norm = T;

    fn norm(&self) -> Self::Norm {
        self.norm_squared().sqrt()
    }

    fn norm_squared(&self) -> Self::Norm {
        self.x().clone() * self.x().clone()
            + self.y().clone() * self.y().clone()
            + self.z().clone() * self.z().clone()
            + self.w().clone() * self.w().clone()
    }

    fn scale_mut(&mut self, n: Self::Norm) {
        *self.x_mut() *= n.clone();
        *self.y_mut() *= n.clone();
        *self.z_mut() *= n.clone();
        *self.w_mut() *= n;
    }

    fn unscale_mut(&mut self, n: Self::Norm) {
        *self.x_mut() /= n.clone();
        *self.y_mut() /= n.clone();
        *self.z_mut() /= n.clone();
        *self.w_mut() /= n;
    }
}
