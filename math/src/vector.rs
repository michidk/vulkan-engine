use std::{
    marker::PhantomData,
    ops::{Add, AddAssign, Div, Mul, Rem, Sub},
};

use crate::{
    angle::{Angle, AngleConst},
    matrix::{Matrix, Owned},
    norm::Normed,
    scalar::Zero,
    storage::{Storage, StorageMut},
    unit::Unit,
};
use crate::{
    scalar::{One, Scalar},
    storage::ArrayStorage,
};

type RowVector<S, T, const C: usize> = Matrix<S, T, 1, C>;
type ColVector<S, T, const R: usize> = Matrix<S, T, R, 1>;

pub type Vec2<T, S = Owned<T, 2, 1>> = ColVector<S, T, 2>;
pub type Vec3<T, S = Owned<T, 3, 1>> = ColVector<S, T, 3>;
pub type Vec4<T, S = Owned<T, 4, 1>> = ColVector<S, T, 4>;

pub struct VecLen0 {}
pub struct VecLen1 {}
pub struct VecLen2 {}
pub struct VecLen3 {}
pub struct VecLen4 {}

pub struct Greater;
pub trait VecLenCmp<T> {
    type Cmp;
}

macro_rules! impl_vec_len_gt {
    ( $($vec:ty => $( $len:ty )+  ;)+ ) => {
        $($(
            impl<T, S> VecLenCmp<$len> for $vec {
                type Cmp = Greater;
            }
        )+)+
    };
}

impl_vec_len_gt! {
    Vec2<T, S> => VecLen0 VecLen1;
    Vec3<T, S> => VecLen0 VecLen1 VecLen2;
    Vec4<T, S> => VecLen0 VecLen1 VecLen2 VecLen3;
}

impl<T, const R: usize> ColVector<Owned<T, R, 1>, T, R> {
    pub fn unit_x() -> Self
    where
        Self: VecLenCmp<VecLen0, Cmp = Greater>,
        T: Clone + Zero + One + Mul<T, Output = T>,
    {
        let mut zero = Matrix::zero();
        unsafe {
            *zero.storage.get_unchecked_mut(0, 0) = T::one();
        };
        zero
    }

    pub fn unit_y() -> Self
    where
        Self: VecLenCmp<VecLen1, Cmp = Greater>,
        T: Clone + Zero + One + Mul<T, Output = T>,
    {
        let mut zero = Matrix::zero();
        unsafe {
            *zero.storage.get_unchecked_mut(1, 0) = T::one();
        };
        zero
    }

    pub fn unit_z() -> Self
    where
        Self: VecLenCmp<VecLen2, Cmp = Greater>,
        T: Clone + Zero + One + Mul<T, Output = T>,
    {
        let mut zero = Matrix::zero();
        unsafe {
            *zero.storage.get_unchecked_mut(2, 0) = T::one();
        };
        zero
    }

    pub fn unit_w() -> Self
    where
        Self: VecLenCmp<VecLen3, Cmp = Greater>,
        T: Clone + Zero + One + Mul<T, Output = T>,
    {
        let mut zero = Matrix::zero();
        unsafe {
            *zero.storage.get_unchecked_mut(3, 0) = T::one();
        };
        zero
    }
}

impl<T, S, const R: usize> ColVector<S, T, R>
where
    S: Storage<T, R, 1>,
{
    pub fn x(&self) -> &T
    where
        Self: VecLenCmp<VecLen0, Cmp = Greater>,
        T: Clone + Zero + One + Mul<T, Output = T>,
    {
        unsafe { self.storage.get_unchecked(0, 0) }
    }

    pub fn y(&self) -> &T
    where
        Self: VecLenCmp<VecLen1, Cmp = Greater>,
        T: Clone + Zero + One + Mul<T, Output = T>,
    {
        unsafe { self.storage.get_unchecked(1, 0) }
    }

    pub fn z(&self) -> &T
    where
        Self: VecLenCmp<VecLen2, Cmp = Greater>,
        T: Clone + Zero + One + Mul<T, Output = T>,
    {
        unsafe { self.storage.get_unchecked(2, 0) }
    }

    pub fn w(&self) -> &T
    where
        Self: VecLenCmp<VecLen3, Cmp = Greater>,
        T: Clone + Zero + One + Mul<T, Output = T>,
    {
        unsafe { self.storage.get_unchecked(3, 0) }
    }
}

impl<S, T, const R: usize> ColVector<S, T, R> {
    pub fn dot_product<RS, RT>(&self, rhs: &ColVector<RS, RT, R>) -> T
    where
        T: Clone + Zero + Mul<RT, Output = T> + AddAssign<T>,
        S: Storage<T, R, 1>,
        RT: Clone,
        RS: Storage<RT, R, 1>,
    {
        let mut value = T::zero();

        for idx in 0..R {
            value += unsafe {
                self.storage.get_unchecked(idx, 0).clone()
                    * rhs.storage.get_unchecked(idx, 0).clone()
            };
        }

        value
    }
}

impl<S, const R: usize> Normed for ColVector<S, f32, R>
where
    S: Storage<f32, R, 1> + StorageMut<f32, R, 1>,
{
    type Norm = f32;

    fn norm(&self) -> Self::Norm {
        let mut value = 0.0;
        for row_idx in 0..R {
            value += unsafe { *self.storage.get_unchecked(row_idx, 0) }.powi(2);
        }
        value.sqrt()
    }

    fn norm_squared(&self) -> Self::Norm {
        self.norm().powi(2)
    }

    fn scale_mut(&mut self, n: Self::Norm) {
        for row_idx in 0..R {
            unsafe { *self.storage.get_unchecked_mut(row_idx, 0) *= n };
        }
    }

    fn unscale_mut(&mut self, n: Self::Norm) {
        for row_idx in 0..R {
            unsafe { *self.storage.get_unchecked_mut(row_idx, 0) /= n };
        }
    }
}

impl<T> Vec3<T> {
    pub const fn new(x: T, y: T, z: T) -> Self {
        Self::from_storage(ArrayStorage { data: [[x, y, z]] })
    }
}

impl<T, S> Vec3<T, S> {
    pub fn cross_product<RT, RS>(&self, rhs: &Vec3<RT, RS>) -> Vec3<T>
    where
        T: Clone + Mul<RT, Output = T> + Sub<T, Output = T>,
        S: Storage<T, 3, 1>,
        RT: Clone,
        RS: Storage<RT, 3, 1>,
    {
        let (a1, a2, a3) = unsafe {
            (
                self.storage.get_unchecked(0, 0),
                self.storage.get_unchecked(1, 0),
                self.storage.get_unchecked(2, 0),
            )
        };
        let (b1, b2, b3) = unsafe {
            (
                rhs.storage.get_unchecked(0, 0),
                rhs.storage.get_unchecked(1, 0),
                rhs.storage.get_unchecked(2, 0),
            )
        };

        let s1 = a2.clone() * b3.clone() - a3.clone() * b2.clone();
        let s2 = a3.clone() * b1.clone() - a1.clone() * b3.clone();
        let s3 = a1.clone() * b2.clone() - a2.clone() * b1.clone();

        Vec3::new(s1, s2, s3)
    }
}

impl<T> Vec4<T> {
    pub const fn new(x: T, y: T, z: T, w: T) -> Self {
        Self::from_storage(ArrayStorage {
            data: [[x, y, z, w]],
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::test_util::MatrixCmp;

    use super::*;

    #[test]
    fn vec_mul() {
        type RVec4<T> = RowVector<Owned<T, 1, 4>, T, 4>;

        let rv1: RVec4<f32> = [[1.0], [0.0], [0.0], [0.0]].into();

        let cv1: Vec4<f32> = [1.0, 1.0, 1.0, 1.0].into();

        let r1: Matrix<Owned<f32, 1, 1>, f32, 1, 1> = &rv1 * &cv1;

        let res = Matrix::from_storage(ArrayStorage { data: [[1.0]] });

        assert_eq!(r1, res);
    }

    #[test]
    fn vec_dot_product() {
        let cv1: Vec3<f32> = [1.0, 1.0, 1.0].into();
        let cv2: Vec3<f32> = [1.0, 1.0, 1.0].into();

        assert_eq!(cv1.dot_product(&cv2), 3.0);
    }

    #[test]
    fn vec_check_unit_impls() {
        // Test if all units are implemented for the types.
        // The test is considered as "ok" if it compiles.
        let _: Vec2<f32> = Vec2::unit_x();
        let _: Vec2<f32> = Vec2::unit_y();

        let _: Vec3<f32> = Vec3::unit_x();
        let _: Vec3<f32> = Vec3::unit_y();
        let _: Vec3<f32> = Vec3::unit_z();

        let _: Vec4<f32> = Vec4::unit_x();
        let _: Vec4<f32> = Vec4::unit_y();
        let _: Vec4<f32> = Vec4::unit_z();
        let _: Vec4<f32> = Vec4::unit_w();
    }

    #[test]
    fn vec3_cross_product() {
        let a = Vec3::new(1.0, 2.0, -3.0);
        let b = Vec3::new(-6.0, 7.0, 0.2);

        assert_eq!(a.cross_product(&b), Vec3::new(21.4, 17.8, 19.0));
    }

    #[test]
    fn vec3_unit() {
        let vec3 = Vec3::new(12.1, 234.1, -1234.5);
        let is = Unit::new_normalize(vec3);
        let should = Vec3::new(
            0.009633733931462902,
            0.1839963563273617,
            -0.9828797139166076,
        );

        MatrixCmp::<f32>::DEFAULT.eq(&is, &should);
    }
}
