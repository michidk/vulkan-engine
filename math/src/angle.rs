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

impl Angle<f32> {
    const PI_180: f32 = ::std::f32::consts::PI / 180.0;

    pub fn from_rad(radians: f32) -> Self {
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
