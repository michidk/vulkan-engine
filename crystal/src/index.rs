use crate::matrix::Matrix;

pub trait MatrixIndex<T: ?Sized> {
    type Output: ?Sized;

    fn get(self, matrix: &T) -> Option<&Self::Output>;
    fn get_mut(self, matrix: &mut T) -> Option<&mut Self::Output>;
    unsafe fn get_unchecked(self, matrix: *const T) -> *const Self::Output;
    unsafe fn get_unchecked_mut(self, matrix: *mut T) -> *mut Self::Output;
}

impl<T, const R: usize, const C: usize> MatrixIndex<Matrix<T, R, C>> for (usize, usize) {
    type Output = T;

    fn get(self, matrix: &Matrix<T, R, C>) -> Option<&Self::Output> {
        matrix.data.get(self.1)?.get(self.0)
    }

    fn get_mut(self, matrix: &mut Matrix<T, R, C>) -> Option<&mut Self::Output> {
        matrix.data.get_mut(self.1)?.get_mut(self.0)
    }

    unsafe fn get_unchecked(self, matrix: *const Matrix<T, R, C>) -> *const Self::Output {
        (*matrix).data.get_unchecked(self.1).get_unchecked(self.0)
    }

    unsafe fn get_unchecked_mut(self, matrix: *mut Matrix<T, R, C>) -> *mut Self::Output {
        (*matrix)
            .data
            .get_unchecked_mut(self.1)
            .get_unchecked_mut(self.0)
    }
}
