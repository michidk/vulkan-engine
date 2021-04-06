use std::ops::Mul;

use crate::{
    matrix::Matrix,
    scalar::{One, Zero},
};

pub type MatN<T, const SIDE: usize> = Matrix<T, SIDE, SIDE>;

impl<T, const SIDE: usize> MatN<T, SIDE>
where
    T: Clone + Zero + One,
{
    pub fn identity() -> Self {
        let mut zero = Self::zero();

        for idx in 0..SIDE {
            unsafe { *zero.get_unchecked_mut((idx, idx)) = T::one() };
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
        let m: MatN<f32, 4> = &Matrix::identity() * 2.0f32;

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
