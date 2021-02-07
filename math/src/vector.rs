use std::{
    marker::PhantomData,
    ops::{Add, AddAssign, Mul},
};

use crate::{
    matrix::{Matrix, Owned},
    scalar::Zero,
    storage::{Storage, StorageMut},
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

impl<T> Vec3<T> {
    pub fn unit_x() -> Self
    where
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
        T: Clone + Zero + One + Mul<T, Output = T>,
    {
        let mut zero = Matrix::zero();
        unsafe {
            *zero.storage.get_unchecked_mut(2, 0) = T::one();
        };
        zero
    }
}

impl<T> Vec4<T> {
    pub fn unit_x() -> Self
    where
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
        T: Clone + Zero + One + Mul<T, Output = T>,
    {
        let mut zero = Matrix::zero();
        unsafe {
            *zero.storage.get_unchecked_mut(3, 0) = T::one();
        };
        zero
    }
}

impl<T> Vec3<T> {
    pub const fn new(x: T, y: T, z: T) -> Self {
        Self::from_storage(ArrayStorage { data: [[x, y, z]] })
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
}
