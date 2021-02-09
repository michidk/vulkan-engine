use crate::angle::Angle;

use std::ops::Mul;

use crate::{
    matn::MatN,
    matrix::{Matrix, Owned},
    scalar::{One, Zero},
    storage::{ArrayStorage, Storage, StorageMut},
    vector::Vec3,
};

pub type Mat4<T, S = Owned<T, 4, 4>> = MatN<T, S, 4>;

impl<T> Mat4<T> {
    #[rustfmt::skip]
    pub fn new(
        c0r0: T, c1r0: T, c2r0: T, c3r0: T,
        c0r1: T, c1r1: T, c2r1: T, c3r1: T,
        c0r2: T, c1r2: T, c2r2: T, c3r2: T,
        c0r3: T, c1r3: T, c2r3: T, c3r3: T,
    ) -> Self {
        Self::from_storage(ArrayStorage {
            data: [
                [c0r0, c0r1, c0r2, c0r3],
                [c1r0, c1r1, c1r2, c1r3],
                [c2r0, c2r1, c2r2, c2r3],
                [c3r0, c3r1, c3r2, c3r3],
            ],
        })
    }

    pub fn new_scaling(factor: T) -> Self
    where
        T: Clone + Zero + One + Mul<T, Output = T>,
    {
        let mut matrix = &Matrix::identity() * factor;
        unsafe { *matrix.storage.get_unchecked_mut(3, 3) = T::one() };
        matrix
    }

    pub fn new_translate(direction: Vec3<T>) -> Self
    where
        T: Clone + Zero + One + Mul<T, Output = T>,
    {
        let mut matrix = Matrix::identity();
        unsafe {
            *matrix.storage.get_unchecked_mut(0, 3) = direction.storage.get_unchecked(0, 0).clone();
            *matrix.storage.get_unchecked_mut(1, 3) = direction.storage.get_unchecked(1, 0).clone();
            *matrix.storage.get_unchecked_mut(2, 3) = direction.storage.get_unchecked(2, 0).clone();
        };

        matrix
    }
}

impl Mat4<f32> {
    pub fn new_rotation_x(angle: Angle<f32>) -> Self {
        let rad = angle.to_rad_clamped();
        let sin = rad.sin();
        let cos = rad.cos();

        Self::from_storage(ArrayStorage {
            data: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, cos, -sin, 0.0],
                [0.0, sin, cos, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        })
    }

    pub fn new_rotation_y(angle: Angle<f32>) -> Self {
        let rad = angle.to_rad_clamped();
        let sin = rad.sin();
        let cos = rad.cos();

        Self::from_storage(ArrayStorage {
            data: [
                [cos, 0.0, sin, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [-sin, 0.0, cos, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        })
    }

    pub fn new_rotation_z(angle: Angle<f32>) -> Self {
        let rad = angle.to_rad_clamped();
        let sin = rad.sin();
        let cos = rad.cos();

        Self::from_storage(ArrayStorage {
            data: [
                [cos, sin, 0.0, 0.0],
                [-sin, cos, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mat4_translate() {
        let m = Mat4::new_translate(Vec3::new(1.0, 0.0, 1.0));

        println!("{:#?}", m);
    }
}
