use std::{
    fmt,
    marker::PhantomData,
    ops::{Add, AddAssign, Mul, Sub},
};

use crate::{
    scalar::{One, Scalar, Zero},
    storage::{Allocator, ArrayStorage, DefaultAllocator, DefaultStorage, Storage, StorageMut},
};

pub type Owned<T, const R: usize, const C: usize> =
    <DefaultAllocator as Allocator<T, R, C>>::Buffer;

#[repr(C)]
pub struct Matrix<S, T: Scalar, const R: usize, const C: usize> {
    pub(crate) storage: S,
    pub(crate) _marker: PhantomData<T>,
}

impl<S, T, const R: usize, const C: usize> Matrix<S, T, R, C> {
    pub(crate) const fn from_storage(storage: S) -> Self {
        Self {
            storage,
            _marker: PhantomData,
        }
    }

    unsafe fn new_uninitialized<A>() -> Matrix<A::Buffer, T, R, C>
    where
        A: Allocator<T, R, C>,
    {
        Matrix::from_storage(A::allocate_unitialized())
    }

    pub const fn shape(&self) -> (usize, usize) {
        (R, C)
    }

    pub fn into_col_arr(self) -> [[T; R]; C]
    where
        S: Into<[[T; R]; C]>,
    {
        self.into()
    }
}

impl<T, const R: usize, const C: usize> Matrix<Owned<T, R, C>, T, R, C> {
    unsafe fn new_uninitialized_default() -> Matrix<Owned<T, R, C>, T, R, C> {
        Self::new_uninitialized::<DefaultAllocator>()
    }
}

impl<S, T, const R: usize, const C: usize> Clone for Matrix<S, T, R, C>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        Self::from_storage(self.storage.clone())
    }
}

impl<S, T, const R: usize, const C: usize> Copy for Matrix<S, T, R, C> where S: Copy {}

impl<S, T, const R: usize, const C: usize> fmt::Debug for Matrix<S, T, R, C>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Matrix")
            .field("shape", &self.shape())
            .field("data", &self.storage)
            .finish()
    }
}

impl<T, const R: usize, const C: usize> Default for Matrix<ArrayStorage<T, R, C>, T, R, C>
where
    T: Default,
{
    fn default() -> Self {
        let mut matrix: Matrix<Owned<T, R, C>, T, R, C> =
            unsafe { Matrix::new_uninitialized_default() };

        for col in 0..C {
            for row in 0..R {
                unsafe { *matrix.storage.get_unchecked_mut(row, col) = T::default() };
            }
        }

        matrix
    }
}

impl<S, T, const R: usize, const C: usize> PartialEq for Matrix<S, T, R, C>
where
    S: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.storage.eq(&other.storage)
    }
}

impl<T, const R: usize, const C: usize> From<[[T; R]; C]>
    for Matrix<ArrayStorage<T, R, C>, T, R, C>
{
    fn from(data: [[T; R]; C]) -> Self {
        Self {
            storage: ArrayStorage { data },
            _marker: PhantomData,
        }
    }
}

impl<T, const R: usize> From<[T; R]> for Matrix<ArrayStorage<T, R, 1>, T, R, 1> {
    fn from(data: [T; R]) -> Self {
        Self {
            storage: ArrayStorage { data: [data] },
            _marker: PhantomData,
        }
    }
}

impl<S, T, const R: usize, const C: usize> From<Matrix<S, T, R, C>> for [[T; R]; C]
where
    S: Into<[[T; R]; C]>,
{
    fn from(value: Matrix<S, T, R, C>) -> Self {
        value.storage.into()
    }
}

impl<S, T, const R: usize> From<Matrix<S, T, R, 1>> for [T; R]
where
    S: Into<[T; R]>,
{
    fn from(value: Matrix<S, T, R, 1>) -> Self {
        value.storage.into()
    }
}

impl<T, const R: usize, const C: usize> Zero for Matrix<Owned<T, R, C>, T, R, C>
where
    T: Zero,
{
    fn zero() -> Self {
        let mut matrix: Matrix<Owned<T, R, C>, T, R, C> =
            unsafe { Matrix::new_uninitialized_default() };

        for col in 0..C {
            for row in 0..R {
                unsafe { *matrix.storage.get_unchecked_mut(row, col) = T::zero() };
            }
        }

        matrix
    }
}

impl<T, const R: usize, const C: usize> One for Matrix<Owned<T, R, C>, T, R, C>
where
    T: One,
{
    fn one() -> Self {
        let mut matrix: Matrix<Owned<T, R, C>, T, R, C> =
            unsafe { Matrix::new_uninitialized_default() };

        for col in 0..C {
            for row in 0..R {
                unsafe { *matrix.storage.get_unchecked_mut(row, col) = T::one() };
            }
        }

        matrix
    }
}

impl<SS, RS, ST, RT, const R: usize, const C: usize> AddAssign<Matrix<RS, RT, R, C>>
    for Matrix<SS, ST, R, C>
where
    ST: AddAssign<RT>,
    SS: StorageMut<ST, R, C>,
    RT: Clone,
    RS: Storage<RT, R, C>,
{
    fn add_assign(&mut self, rhs: Matrix<RS, RT, R, C>) {
        for col_idx in 0..C {
            for row_idx in 0..R {
                unsafe {
                    *self.storage.get_unchecked_mut(row_idx, col_idx) +=
                        rhs.storage.get_unchecked(row_idx, col_idx).clone();
                };
            }
        }
    }
}

impl<'a, 'b, SS, RS, ST, RT, const R: usize, const C: usize> Sub<&'a Matrix<RS, RT, R, C>>
    for &'b Matrix<SS, ST, R, C>
where
    ST: Clone + Sub<RT, Output = ST>,
    SS: Storage<ST, R, C>,
    RT: Clone,
    RS: Storage<RT, R, C>,
    DefaultAllocator: Allocator<ST, R, C>,
{
    type Output = Matrix<Owned<ST, R, C>, ST, R, C>;

    fn sub(self, rhs: &'a Matrix<RS, RT, R, C>) -> Self::Output {
        let mut matrix: Self::Output = unsafe { Self::Output::new_uninitialized_default() };

        for col_idx in 0..C {
            for row_idx in 0..R {
                unsafe {
                    *matrix.storage.get_unchecked_mut(row_idx, col_idx) =
                        self.storage.get_unchecked(row_idx, col_idx).clone()
                            - rhs.storage.get_unchecked(row_idx, col_idx).clone();
                };
            }
        }

        matrix
    }
}

impl<'a, 'b, SS, RS, ST, RT, const SR: usize, const SHARED: usize, const RC: usize>
    Mul<&'a Matrix<RS, RT, SHARED, RC>> for &'b Matrix<SS, ST, SR, SHARED>
where
    ST: Clone + Zero + AddAssign<ST> + Mul<RT, Output = ST>,
    SS: Storage<ST, SR, SHARED>,
    RT: Clone,
    RS: Storage<RT, SHARED, RC>,
    DefaultAllocator: Allocator<ST, SR, RC>,
{
    type Output = Matrix<Owned<ST, SR, RC>, ST, SR, RC>;

    fn mul(self, rhs: &'a Matrix<RS, RT, SHARED, RC>) -> Self::Output {
        let mut matrix: Self::Output = unsafe { Self::Output::new_uninitialized_default() };

        for col_idx in 0..RC {
            for row_idx in 0..SR {
                let mut value = ST::zero();
                for idx in 0..SHARED {
                    value += unsafe {
                        self.storage.get_unchecked(row_idx, idx).clone()
                            * rhs.storage.get_unchecked(idx, col_idx).clone()
                    };
                }

                unsafe { *matrix.storage.get_unchecked_mut(row_idx, col_idx) = value };
            }
        }

        matrix
    }
}

impl<'a, S, T, const R: usize, const C: usize> Mul<T> for &'a Matrix<S, T, R, C>
where
    T: Clone + Mul<T, Output = T>,
    S: Storage<T, R, C>,
    DefaultAllocator: Allocator<T, R, C>,
{
    type Output = Matrix<Owned<T, R, C>, T, R, C>;

    fn mul(self, rhs: T) -> Self::Output {
        let mut matrix: Self::Output = unsafe { Self::Output::new_uninitialized_default() };

        for col_idx in 0..C {
            for row_idx in 0..R {
                unsafe {
                    *matrix.storage.get_unchecked_mut(row_idx, col_idx) =
                        self.storage.get_unchecked(row_idx, col_idx).clone() * rhs.clone()
                };
            }
        }

        matrix
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Mat4<T> = Matrix<Owned<T, 4, 4>, T, 4, 4>;
    type Vec4<T> = Matrix<Owned<T, 4, 1>, T, 4, 1>;

    #[test]
    fn mat4_mul() {
        let m1: Mat4<f32> = [
            [1.0, 2.0, 3.0, 4.0],
            [2.0, 3.0, 4.0, 3.0],
            [3.0, 4.0, 3.0, 2.0],
            [4.0, 3.0, 2.0, 1.0],
        ]
        .into();

        let m2: Mat4<f32> = Mat4::identity();

        let m3: Mat4<f32> = &m1 * &(&m2 * 2.0);

        let res: Mat4<f32> = [
            [2.0, 4.0, 6.0, 8.0],
            [4.0, 6.0, 8.0, 6.0],
            [6.0, 8.0, 6.0, 4.0],
            [8.0, 6.0, 4.0, 2.0],
        ]
        .into();

        assert_eq!(m3, res);
    }
}
