use std::ops::Mul;

use crate::{
    matrix::{Matrix, Owned},
    scalar::{One, Zero},
    storage::{ArrayStorage, StorageMut},
};

pub type MatN<T, S, const SIDE: usize> = Matrix<S, T, SIDE, SIDE>;

impl<S, T, const SIDE: usize> MatN<T, S, SIDE> {}

impl<T, const SIDE: usize> MatN<T, Owned<T, SIDE, SIDE>, SIDE>
where
    T: Clone + Zero + One,
{
    pub fn identity() -> Self {
        let mut zero = Matrix::zero();

        for idx in 0..SIDE {
            unsafe { *zero.storage.get_unchecked_mut(idx, idx) = T::one() };
        }

        zero
    }
}

#[cfg(test)]
mod tests {
    use crate::mat4::Mat4;

    use super::*;

    #[test]
    fn matn_identity() {
        let m: MatN<f32, _, 4> = &Matrix::identity() * 2.0;

        let res: Mat4<f32> = [
            [2.0, 0.0, 0.0, 0.0],
            [0.0, 2.0, 0.0, 0.0],
            [0.0, 0.0, 2.0, 0.0],
            [0.0, 0.0, 0.0, 2.0],
        ]
        .into();

        assert_eq!(m, res);
    }
}
