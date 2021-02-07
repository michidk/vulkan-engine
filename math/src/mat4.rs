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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mat4_translate() {
        let m = Mat4::new_translate(Vec3::new(1.0, 0.0, 1.0));

        println!("{:#?}", m);
    }
}
