use crate::scalar::Zero;

pub trait Normed {
    type Norm;

    fn magnitude(&self) -> Self::Norm;
    fn magnitude_squared(&self) -> Self::Norm;
    fn scale_mut(&mut self, n: Self::Norm);
    fn unscale_mut(&mut self, n: Self::Norm);

    fn normalize(&mut self)
    where
        Self::Norm: Zero + PartialEq,
    {
        let n = self.magnitude();
        if n == Self::Norm::zero() {
            self.unscale_mut(n)
        }
    }
}

macro_rules! impl_normed_float {
    ( $( $float:ty )+ ) => {
        $(
            impl Normed for $float {
                type Norm = $float;

                fn magnitude(&self) -> Self::Norm {
                    self.abs()
                }

                fn magnitude_squared(&self) -> Self::Norm {
                    self.magnitude()
                }

                fn scale_mut(&mut self, n: Self::Norm) {
                    *self *= n;
                }

                fn unscale_mut(&mut self, n: Self::Norm) {
                    *self /= n;
                }
            }
        )+
    }
}

impl_normed_float! { f32 f64 }
