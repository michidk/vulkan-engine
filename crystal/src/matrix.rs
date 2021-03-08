use std::{
    fmt,
    ops::{Add, AddAssign, Mul, Neg, Sub},
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

    unsafe fn uninitialized() -> Matrix<T, R, C> {
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

impl<ST, RT, const R: usize, const C: usize> AddAssign<Matrix<RT, R, C>> for Matrix<ST, R, C>
where
    ST: AddAssign<RT>,
    RT: Clone,
{
    fn add_assign(&mut self, rhs: Matrix<RT, R, C>) {
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

impl<'a, T, const R: usize, const C: usize> Mul<T> for &'a Matrix<T, R, C>
where
    T: Clone + Mul<T, Output = T>,
{
    type Output = Matrix<T, R, C>;

    fn mul(self, rhs: T) -> Self::Output {
        let mut matrix = unsafe { Matrix::uninitialized() };

        for col_idx in 0..C {
            for row_idx in 0..R {
                unsafe {
                    *matrix.get_unchecked_mut((row_idx, col_idx)) =
                        self.get_unchecked((row_idx, col_idx)).clone() * rhs.clone()
                };
            }
        }

        matrix
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
