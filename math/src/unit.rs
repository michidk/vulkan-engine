use std::{fmt, ops::Deref};

use crate::norm::Normed;

#[repr(transparent)]
pub struct Unit<T> {
    value: T,
}

impl<T> Unit<T>
where
    T: Normed,
    T::Norm: Clone,
{
    pub fn new_normalize(value: T) -> Self {
        Self::new_and_get(value).0
    }

    pub fn new_and_get(mut value: T) -> (Self, T::Norm) {
        let n = value.norm();
        value.unscale_mut(n.clone());
        (Unit { value }, n)
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
