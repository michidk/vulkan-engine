use std::{fmt, ops::Sub};

use crate::{matrix::Matrix, scalar::Abs, storage::Storage};

#[derive(Debug, Clone)]
pub struct MatrixCmp<T> {
    error_margin: T,
}

impl<T> MatrixCmp<T>
where
    T: fmt::Debug + Clone + PartialOrd + Abs<Output = T> + Sub<T, Output = T>,
{
    pub fn eq_margin<OS, TS, const R: usize, const C: usize>(
        &self,
        mat_one: &Matrix<OS, T, R, C>,
        mat_two: &Matrix<TS, T, R, C>,
        error_margin: T,
    ) where
        OS: Storage<T, R, C>,
        TS: Storage<T, R, C>,
    {
        for col_idx in 0..R {
            for row_idx in 0..C {
                let (v1, v2) = unsafe {
                    (
                        mat_one.storage.get_unchecked(row_idx, col_idx),
                        mat_two.storage.get_unchecked(row_idx, col_idx),
                    )
                };
                let diff_abs = (v1.clone() - v2.clone()).abs();

                assert!(
                    diff_abs <= error_margin,
                    "is: {:?} should: {:?} (+- {:?}) @ ({}, {})",
                    v1,
                    v2,
                    error_margin,
                    row_idx,
                    col_idx
                );
            }
        }
    }

    pub fn eq<OS, TS, const R: usize, const C: usize>(
        &self,
        mat_one: &Matrix<OS, T, R, C>,
        mat_two: &Matrix<TS, T, R, C>,
    ) where
        OS: Storage<T, R, C>,
        TS: Storage<T, R, C>,
    {
        self.eq_margin(mat_one, mat_two, self.error_margin.clone())
    }
}

impl MatrixCmp<f32> {
    pub const DEFAULT: Self = Self { error_margin: 1e6 };
}

impl MatrixCmp<f64> {
    pub const DEFAULT: Self = Self { error_margin: 1e6 };
}

impl Default for MatrixCmp<f32> {
    fn default() -> Self {
        Self { error_margin: 1e6 }
    }
}

impl Default for MatrixCmp<f64> {
    fn default() -> Self {
        Self { error_margin: 1e6 }
    }
}
