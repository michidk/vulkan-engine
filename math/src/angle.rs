use std::fmt;

pub trait ToAngle: Sized {
    fn rad(self) -> Angle<Self>;
    fn deg(self) -> Angle<Self>;
}

impl ToAngle for f32 {
    fn rad(self) -> Angle<Self> {
        Angle::from_rad(self)
    }

    fn deg(self) -> Angle<Self> {
        Angle::from_deg(self)
    }
}

pub struct Angle<T> {
    radians: T,
}

impl<T: fmt::Debug> fmt::Debug for Angle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Angle")
            .field(&self.radians)
            .finish()
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

impl Angle<f32> {
    const PI_180: f32 = ::std::f32::consts::PI / 180.0;

    pub fn from_rad(radians: f32) -> Self {
        assert!(!radians.is_infinite(), "Radians is infinite");
        assert!(!radians.is_nan(), "Radians is NaN");
        Self { radians }
    }

    pub fn from_deg(degree: f32) -> Self {
        Self::from_rad(degree * Self::PI_180)
    }

    pub fn to_rad(&self) -> f32 {
        self.radians
    }

    pub fn to_deg(&self) -> f32 {
        self.radians / Self::PI_180
    }

    pub fn to_rad_clamped(&self) -> f32 {
        self.radians % std::f32::consts::TAU
    }

    pub fn to_deg_clamped(&self) -> f32 {
        self.to_deg() % 360.0
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angle_degrad() {
        assert_eq!(Angle::from_deg(90.0).to_rad(), 1.5707964);
        assert_eq!(Angle::from_rad(0.7853982).to_deg(), 45.0);
    }
}
