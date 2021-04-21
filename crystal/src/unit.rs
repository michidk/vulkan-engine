use std::{fmt, ops::Deref};

use crate::norm::Normed;
use crate::scalar::Zero;

#[repr(transparent)]
pub struct Unit<T> {
    value: T,
}

impl<T> Unit<T>
where
    T: Normed,
    T::Norm: Clone + Zero + PartialEq,
{
    pub fn new_normalize(mut value: T) -> Self {
        value.normalize();

        Self { value }
    }
}

impl<T> Unit<T> {
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T: fmt::Debug> fmt::Debug for Unit<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Unit").field(&self.value).finish()
    }
}

impl<T: Clone> Clone for Unit<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
        }
    }
}

impl<T: Copy> Copy for Unit<T> {}

impl<T> Deref for Unit<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> AsRef<T> for Unit<T> {
    fn as_ref(&self) -> &T {
        &self.value
    }
}
