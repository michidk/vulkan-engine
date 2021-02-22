pub trait Normed {
    type Norm;

    fn norm(&self) -> Self::Norm;
    fn norm_squared(&self) -> Self::Norm;
    fn scale_mut(&mut self, n: Self::Norm);
    fn unscale_mut(&mut self, n: Self::Norm);
}

macro_rules! impl_normed_float {
    ( $( $float:ty )+ ) => {
        $(
            impl Normed for $float {
                type Norm = $float;

                fn norm(&self) -> Self::Norm {
                    self.abs()
                }

                fn norm_squared(&self) -> Self::Norm {
                    self.norm()
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
