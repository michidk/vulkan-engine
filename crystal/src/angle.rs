use std::{
    fmt,
    ops::{Div, Mul, Neg, Rem},
};

pub trait IntoAngle: Sized {
    fn rad(self) -> Angle<Self>;
    fn deg(self) -> Angle<Self>;
}

impl IntoAngle for f32 {
    fn rad(self) -> Angle<Self> {
        Angle::from_rad(self)
    }

    fn deg(self) -> Angle<Self> {
        Angle::from_deg(self)
    }
}

pub trait AngleConst {
    const PI_180: Self;
    const TAU: Self;
}

pub struct Angle<T> {
    radians: T,
}

impl<T: fmt::Debug> fmt::Debug for Angle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Angle").field(&self.radians).finish()
    }
}

impl<T: Default> Default for Angle<T> {
    fn default() -> Self {
        Self {
            radians: T::default(),
        }
    }
}

impl<T: Copy> Copy for Angle<T> {}

impl<T: Clone> Clone for Angle<T> {
    fn clone(&self) -> Self {
        Self {
            radians: self.radians.clone(),
        }
    }
}

impl<T: PartialEq<T>> PartialEq<Self> for Angle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.radians.eq(&other.radians)
    }
}

impl<T: Eq> Eq for Angle<T> {}

impl<T: PartialOrd<T>> PartialOrd<Self> for Angle<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.radians.partial_cmp(&other.radians)
    }
}

impl<T: Ord> Ord for Angle<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.radians.cmp(&other.radians)
    }
}

impl<T: Neg<Output = T>> Neg for Angle<T> {
    type Output = Angle<T>;

    fn neg(self) -> Self::Output {
        Self {
            radians: -self.radians,
        }
    }
}

impl<T> Angle<T>
where
    T: Clone + AngleConst + Mul<T, Output = T> + Div<T, Output = T> + Rem<T, Output = T>,
{
    pub fn from_rad(radians: T) -> Self {
        Self { radians }
    }

    pub fn from_deg(degree: T) -> Self {
        Self::from_rad(degree * T::PI_180)
    }

    pub fn to_rad(&self) -> T {
        self.radians.clone()
    }

    pub fn to_deg(&self) -> T {
        self.radians.clone() / T::PI_180
    }

    pub fn to_rad_clamped(&self) -> T {
        self.radians.clone() % T::TAU
    }

    pub fn to_deg_clamped(&self) -> T {
        Self::from_rad(self.to_rad_clamped()).to_deg()
    }
}

impl AngleConst for f32 {
    const PI_180: Self = std::f32::consts::PI / 180.0;
    const TAU: Self = std::f32::consts::TAU;
}

impl AngleConst for f64 {
    const PI_180: Self = std::f64::consts::PI / 180.0;
    const TAU: Self = std::f64::consts::TAU;
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_eq_err {
        ($left:expr, $right:expr, $err:literal $(,)?) => {{
            match (&$left, &$right) {
                (left_val, right_val) => {
                    if ((*left_val - *right_val).abs() > $err) {
                        // The reborrows below are intentional. Without them, the stack slot for the
                        // borrow is initialized even before the values are compared, leading to a
                        // noticeable slow down.
                        panic!(
                            r#"assertion failed: `(left == right)`
  left: `{:?}`,
 right: `{:?}`"#,
                            &*left_val, &*right_val
                        )
                    }
                }
            }
        }};
    }

    #[test]
    fn angle_degrad() {
        // TODO: tests
        assert_eq_err!(Angle::from_deg(90.0f32).to_rad(), 1.570796, 1e-5);
    }
}
