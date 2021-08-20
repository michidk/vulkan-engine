use std::{
    fmt,
    ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign},
};

use crate::{
    index::MatrixIndex,
    scalar::{One, Scalar, Zero},
};

#[repr(C)]
pub struct Matrix<T: Scalar, const R: usize, const C: usize> {
    pub(crate) data: [[T; R]; C],
}

impl<T, const R: usize, const C: usize> Matrix<T, R, C> {
    pub const fn from_data(data: [[T; R]; C]) -> Self {
        Self { data }
    }

    pub const fn shape(&self) -> (usize, usize) {
        (R, C)
    }

    #[allow(clippy::uninit_assumed_init)]
    pub(crate) unsafe fn uninitialized() -> Matrix<T, R, C> {
        Self::from_data(std::mem::MaybeUninit::uninit().assume_init())
    }

    pub fn get<I>(&self, index: I) -> Option<&I::Output>
    where
        I: MatrixIndex<Self>,
    {
        index.get(self)
    }

    pub fn get_mut<I>(&mut self, index: I) -> Option<&mut I::Output>
    where
        I: MatrixIndex<Self>,
    {
        index.get_mut(self)
    }

    pub unsafe fn get_unchecked<I>(&self, index: I) -> &I::Output
    where
        I: MatrixIndex<Self>,
    {
        &*index.get_unchecked(self)
    }

    pub unsafe fn get_unchecked_mut<I>(&mut self, index: I) -> &mut I::Output
    where
        I: MatrixIndex<Self>,
    {
        &mut *index.get_unchecked_mut(self)
    }
}

impl<T, const R: usize, const C: usize> Clone for Matrix<T, R, C>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self::from_data(self.data.clone())
    }
}

impl<T, const R: usize, const C: usize> Copy for Matrix<T, R, C> where T: Copy {}

impl<T, const R: usize, const C: usize> fmt::Debug for Matrix<T, R, C>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Matrix")
            .field("shape", &self.shape())
            .field("data", &self.data)
            .finish()
    }
}

impl<T, const R: usize, const C: usize> Default for Matrix<T, R, C>
where
    T: Default,
{
    fn default() -> Self {
        let mut matrix = unsafe { Self::uninitialized() };

        for col in 0..C {
            for row in 0..R {
                unsafe { *matrix.get_unchecked_mut((row, col)) = T::default() };
            }
        }

        matrix
    }
}

impl<T, const R: usize, const C: usize> PartialEq<Self> for Matrix<T, R, C>
where
    T: PartialEq<T>,
{
    fn eq(&self, other: &Self) -> bool {
        self.data.eq(&other.data)
    }
}

impl<T, const R: usize, const C: usize> From<[[T; R]; C]> for Matrix<T, R, C> {
    fn from(data: [[T; R]; C]) -> Self {
        Self::from_data(data)
    }
}

impl<T, const R: usize> From<[T; R]> for Matrix<T, R, 1> {
    fn from(data: [T; R]) -> Self {
        Self::from_data([data])
    }
}

impl<T, const R: usize, const C: usize> From<Matrix<T, R, C>> for [[T; R]; C] {
    fn from(value: Matrix<T, R, C>) -> Self {
        value.data
    }
}

impl<T, const R: usize, const C: usize> Zero for Matrix<T, R, C>
where
    T: Zero,
{
    fn zero() -> Self {
        let mut matrix = unsafe { Self::uninitialized() };

        for col in 0..C {
            for row in 0..R {
                unsafe { *matrix.get_unchecked_mut((row, col)) = T::zero() };
            }
        }

        matrix
    }
}

impl<T, const R: usize, const C: usize> One for Matrix<T, R, C>
where
    T: One,
{
    fn one() -> Self {
        let mut matrix = unsafe { Matrix::uninitialized() };

        for col in 0..C {
            for row in 0..R {
                unsafe { *matrix.get_unchecked_mut((row, col)) = T::one() };
            }
        }

        matrix
    }
}

impl<'a, ST, RT, const R: usize, const C: usize> AddAssign<&'a Matrix<RT, R, C>>
    for Matrix<ST, R, C>
where
    ST: AddAssign<RT>,
    RT: Clone,
{
    fn add_assign(&mut self, rhs: &'a Matrix<RT, R, C>) {
        for col_idx in 0..C {
            for row_idx in 0..R {
                unsafe {
                    *self.get_unchecked_mut((row_idx, col_idx)) +=
                        rhs.get_unchecked((row_idx, col_idx)).clone();
                };
            }
        }
    }
}

impl<ST, RT, const R: usize, const C: usize> AddAssign<Matrix<RT, R, C>> for Matrix<ST, R, C>
where
    ST: AddAssign<RT>,
    RT: Clone,
{
    fn add_assign(&mut self, rhs: Matrix<RT, R, C>) {
        AddAssign::add_assign(self, &rhs)
    }
}

impl<'a, ST, RT, const R: usize, const C: usize> SubAssign<&'a Matrix<RT, R, C>>
    for Matrix<ST, R, C>
where
    ST: SubAssign<RT>,
    RT: Clone,
{
    fn sub_assign(&mut self, rhs: &'a Matrix<RT, R, C>) {
        for col_idx in 0..C {
            for row_idx in 0..R {
                unsafe {
                    *self.get_unchecked_mut((row_idx, col_idx)) -=
                        rhs.get_unchecked((row_idx, col_idx)).clone();
                };
            }
        }
    }
}

impl<ST, RT, const R: usize, const C: usize> SubAssign<Matrix<RT, R, C>> for Matrix<ST, R, C>
where
    ST: SubAssign<RT>,
    RT: Clone,
{
    fn sub_assign(&mut self, rhs: Matrix<RT, R, C>) {
        SubAssign::sub_assign(self, &rhs)
    }
}

impl<'a, 'b, ST, RT, const R: usize, const C: usize> Sub<&'a Matrix<RT, R, C>>
    for &'b Matrix<ST, R, C>
where
    ST: Clone + Sub<RT, Output = ST>,
    RT: Clone,
{
    type Output = Matrix<ST, R, C>;

    fn sub(self, rhs: &'a Matrix<RT, R, C>) -> Self::Output {
        let mut matrix = unsafe { Matrix::uninitialized() };

        for col_idx in 0..C {
            for row_idx in 0..R {
                unsafe {
                    *matrix.get_unchecked_mut((row_idx, col_idx)) =
                        self.get_unchecked((row_idx, col_idx)).clone()
                            - rhs.get_unchecked((row_idx, col_idx)).clone();
                };
            }
        }

        matrix
    }
}

impl<'a, ST, RT, const R: usize, const C: usize> Sub<&'a Matrix<RT, R, C>> for Matrix<ST, R, C>
where
    ST: Clone + Sub<RT, Output = ST>,
    RT: Clone,
{
    type Output = Matrix<ST, R, C>;

    fn sub(mut self, rhs: &'a Matrix<RT, R, C>) -> Self::Output {
        for col_idx in 0..C {
            for row_idx in 0..R {
                unsafe {
                    *self.get_unchecked_mut((row_idx, col_idx)) =
                        self.get_unchecked((row_idx, col_idx)).clone()
                            - rhs.get_unchecked((row_idx, col_idx)).clone();
                };
            }
        }

        self
    }
}

impl<ST, RT, const R: usize, const C: usize> Sub<Matrix<RT, R, C>> for Matrix<ST, R, C>
where
    ST: Clone + Sub<RT, Output = ST>,
    RT: Clone,
{
    type Output = Matrix<ST, R, C>;

    fn sub(mut self, rhs: Matrix<RT, R, C>) -> Self::Output {
        for col_idx in 0..C {
            for row_idx in 0..R {
                unsafe {
                    *self.get_unchecked_mut((row_idx, col_idx)) =
                        self.get_unchecked((row_idx, col_idx)).clone()
                            - rhs.get_unchecked((row_idx, col_idx)).clone();
                };
            }
        }

        self
    }
}

impl<'a, 'b, ST, RT, const SR: usize, const SHARED: usize, const RC: usize>
    Mul<&'a Matrix<RT, SHARED, RC>> for &'b Matrix<ST, SR, SHARED>
where
    ST: Clone + Zero + AddAssign<ST> + Mul<RT, Output = ST>,
    RT: Clone,
{
    type Output = Matrix<ST, SR, RC>;

    fn mul(self, rhs: &'a Matrix<RT, SHARED, RC>) -> Self::Output {
        let mut matrix = unsafe { Matrix::uninitialized() };

        for col_idx in 0..RC {
            for row_idx in 0..SR {
                let mut value = ST::zero();
                for idx in 0..SHARED {
                    value += unsafe {
                        self.get_unchecked((row_idx, idx)).clone()
                            * rhs.get_unchecked((idx, col_idx)).clone()
                    };
                }

                unsafe { *matrix.get_unchecked_mut((row_idx, col_idx)) = value };
            }
        }

        matrix
    }
}

impl<'a, ST, RT, const SR: usize, const SHARED: usize, const RC: usize>
    Mul<&'a Matrix<RT, SHARED, RC>> for Matrix<ST, SR, SHARED>
where
    ST: Clone + Zero + AddAssign<ST> + Mul<RT, Output = ST>,
    RT: Clone,
{
    type Output = Matrix<ST, SR, RC>;

    fn mul(self, rhs: &'a Matrix<RT, SHARED, RC>) -> Self::Output {
        Mul::mul(&self, rhs)
    }
}

impl<'b, ST, RT, const SR: usize, const SHARED: usize, const RC: usize> Mul<Matrix<RT, SHARED, RC>>
    for &'b Matrix<ST, SR, SHARED>
where
    ST: Clone + Zero + AddAssign<ST> + Mul<RT, Output = ST>,
    RT: Clone,
{
    type Output = Matrix<ST, SR, RC>;

    fn mul(self, rhs: Matrix<RT, SHARED, RC>) -> Self::Output {
        Mul::mul(self, &rhs)
    }
}

impl<ST, RT, const SR: usize, const SHARED: usize, const RC: usize> Mul<Matrix<RT, SHARED, RC>>
    for Matrix<ST, SR, SHARED>
where
    ST: Clone + Zero + AddAssign<ST> + Mul<RT, Output = ST>,
    RT: Clone,
{
    type Output = Matrix<ST, SR, RC>;

    fn mul(self, rhs: Matrix<RT, SHARED, RC>) -> Self::Output {
        Mul::mul(&self, &rhs)
    }
}

impl<'a, T, const R: usize, const C: usize> MulAssign<&'a T> for Matrix<T, R, C>
where
    T: Clone + MulAssign<T>,
{
    fn mul_assign(&mut self, rhs: &'a T) {
        for col_idx in 0..C {
            for row_idx in 0..R {
                unsafe {
                    *self.get_unchecked_mut((row_idx, col_idx)) *= rhs.clone();
                };
            }
        }
    }
}

impl<T, const R: usize, const C: usize> MulAssign<T> for Matrix<T, R, C>
where
    T: Clone + MulAssign<T>,
{
    fn mul_assign(&mut self, rhs: T) {
        MulAssign::mul_assign(self, &rhs)
    }
}

macro_rules! impl_mul_scalar {
    ( $( $scalar:ty )+ ) => {
        $(
            impl<'a, 'b, T, const R: usize, const C: usize> Mul<&'a $scalar> for &'b Matrix<T, R, C>
            where
                T: Clone + Mul<$scalar, Output = T>,
            {
                type Output = Matrix<T, R, C>;

                fn mul(self, rhs: &'a $scalar) -> Self::Output {
                    let mut matrix = unsafe { Matrix::uninitialized() };

                    for col_idx in 0..C {
                        for row_idx in 0..R {
                            unsafe { *matrix.get_unchecked_mut((row_idx, col_idx)) = self.get_unchecked((row_idx, col_idx)).clone() * *rhs };
                        }
                    }

                    matrix
                }
            }

            impl<'a, T, const R: usize, const C: usize> Mul<&'a $scalar> for Matrix<T, R, C>
            where
                T: Clone + MulAssign<$scalar>,
            {
                type Output = Self;

                fn mul(mut self, rhs: &'a $scalar) -> Self::Output {
                    for col_idx in 0..C {
                        for row_idx in 0..R {
                            unsafe { *self.get_unchecked_mut((row_idx, col_idx)) *= *rhs };
                        }
                    }

                    self
                }
            }

            impl<'b, T, const R: usize, const C: usize> Mul<$scalar> for &'b Matrix<T, R, C>
            where
                T: Clone + Mul<$scalar, Output = T>,
            {
                type Output = Matrix<T, R, C>;

                fn mul(self, rhs: $scalar) -> Self::Output {
                    let mut matrix = unsafe { Matrix::uninitialized() };

                    for col_idx in 0..C {
                        for row_idx in 0..R {
                            unsafe { *matrix.get_unchecked_mut((row_idx, col_idx)) = self.get_unchecked((row_idx, col_idx)).clone() * rhs };
                        }
                    }

                    matrix
                }
            }

            impl<T, const R: usize, const C: usize> Mul<$scalar> for Matrix<T, R, C>
            where
                T: Clone + MulAssign<$scalar>,
            {
                type Output = Self;

                fn mul(mut self, rhs: $scalar) -> Self::Output {
                    for col_idx in 0..C {
                        for row_idx in 0..R {
                            unsafe { *self.get_unchecked_mut((row_idx, col_idx)) *= rhs };
                        }
                    }

                    self
                }
            }
        )+
    };
}

impl_mul_scalar! { u8 i8 u16 i16 u32 i32 u64 i64 i128 u128 usize isize f32 f64 }

impl<'a, 'b, T, const R: usize, const C: usize> Div<&'a T> for &'b Matrix<T, R, C>
where
    T: Clone + Div<T, Output = T>,
{
    type Output = Matrix<T, R, C>;

    fn div(self, rhs: &'a T) -> Self::Output {
        let mut matrix = unsafe { Matrix::uninitialized() };

        for col_idx in 0..C {
            for row_idx in 0..R {
                unsafe {
                    *matrix.get_unchecked_mut((row_idx, col_idx)) =
                        self.get_unchecked((row_idx, col_idx)).clone() / rhs.clone()
                };
            }
        }

        matrix
    }
}

impl<'a, T, const R: usize, const C: usize> Div<T> for &'a Matrix<T, R, C>
where
    T: Clone + Div<T, Output = T>,
{
    type Output = Matrix<T, R, C>;

    fn div(self, rhs: T) -> Self::Output {
        Div::div(self, &rhs)
    }
}

impl<T, const R: usize, const C: usize> Div<T> for Matrix<T, R, C>
where
    T: Clone + DivAssign<T>,
{
    type Output = Matrix<T, R, C>;

    fn div(mut self, rhs: T) -> Self::Output {
        for col_idx in 0..C {
            for row_idx in 0..R {
                unsafe {
                    // TODO: keep using DivAssign or Clone + Div ??
                    *self.get_unchecked_mut((row_idx, col_idx)) /= rhs.clone();
                };
            }
        }

        self
    }
}

impl<'a, T, const R: usize, const C: usize> DivAssign<&'a T> for Matrix<T, R, C>
where
    T: Clone + DivAssign<T>,
{
    fn div_assign(&mut self, rhs: &'a T) {
        for col_idx in 0..C {
            for row_idx in 0..R {
                unsafe {
                    *self.get_unchecked_mut((row_idx, col_idx)) /= rhs.clone();
                };
            }
        }
    }
}

impl<T, const R: usize, const C: usize> DivAssign<T> for Matrix<T, R, C>
where
    T: Clone + DivAssign<T>,
{
    fn div_assign(&mut self, rhs: T) {
        DivAssign::div_assign(self, &rhs)
    }
}

impl<'a, T, const R: usize, const C: usize> Neg for &'a Matrix<T, R, C>
where
    T: Clone + Neg<Output = T>,
{
    type Output = Matrix<T, R, C>;

    fn neg(self) -> Self::Output {
        let mut matrix = unsafe { Matrix::uninitialized() };

        for col_idx in 0..C {
            for row_idx in 0..R {
                unsafe {
                    *matrix.get_unchecked_mut((row_idx, col_idx)) =
                        -self.get_unchecked((row_idx, col_idx)).clone();
                };
            }
        }

        matrix
    }
}

impl<T, const R: usize, const C: usize> Neg for Matrix<T, R, C>
where
    T: Clone + Neg<Output = T>,
{
    type Output = Self;

    fn neg(mut self) -> Self::Output {
        for col_idx in 0..C {
            for row_idx in 0..R {
                unsafe {
                    *self.get_unchecked_mut((row_idx, col_idx)) =
                        -self.get_unchecked((row_idx, col_idx)).clone();
                };
            }
        }

        self
    }
}

impl<I, T, const R: usize, const C: usize> Index<I> for Matrix<T, R, C>
where
    I: MatrixIndex<Matrix<T, R, C>>,
{
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        index.get(self).unwrap()
    }
}

impl<I, T, const R: usize, const C: usize> IndexMut<I> for Matrix<T, R, C>
where
    I: MatrixIndex<Matrix<T, R, C>>,
    T: Clone + Neg<Output = T>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        index.get_mut(self).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mat4::Mat4;
    use crate::test_util::MatrixCmp;

    #[test]
    fn mat4_mul() {
        let is: Mat4<f32> = &Mat4::identity() * (Mat4::<f32>::identity() * 2.0f32);
        let should = Mat4::new(
            2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 2.0,
        );

        MatrixCmp::<f32>::DEFAULT.eq(&is, &should);
    }
}
