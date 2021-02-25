pub trait Zero {
    fn zero() -> Self;
}

pub trait One {
    fn one() -> Self;
}

pub trait Abs {
    type Output;

    fn abs(&self) -> Self::Output;
}

pub trait Sin {
    type Output;

    fn sin(&self) -> Self::Output;
}

pub trait Cos {
    type Output;

    fn cos(&self) -> Self::Output;
}

pub trait Sqrt {
    type Output;

    fn sqrt(&self) -> Self::Output;
}

pub trait Scalar {}

macro_rules! impl_nums_zero {
    ( $( $num:ty )+ ) => {
        $(
            impl Zero for $num {
                fn zero() -> Self {
                    0 as $num
                }
            }
        )+
    };
}

macro_rules! impl_nums_one {
    ( $( $num:ty )+ ) => {
        $(
            impl One for $num {
                fn one() -> Self {
                    1 as $num
                }
            }
        )+
    };
}

macro_rules! impl_inums_abs {
    ( $( $num:ty )+ ) => {
        $(
            impl Abs for $num {
                type Output = Self;

                fn abs(&self) -> Self::Output {
                    (*self).abs()
                }
            }
        )+
    };
}

macro_rules! impl_unums_abs {
    ( $( $num:ty )+ ) => {
        $(
            impl Abs for $num {
                type Output = Self;

                fn abs(&self) -> Self::Output {
                    *self
                }
            }
        )+
    };
}

macro_rules! impl_float_sin_cos {
    ( $( $float:ty )+ ) => {
        $(
            impl Sin for $float {
                type Output = Self;

                fn sin(&self) -> Self::Output {
                    (*self).sin()
                }
            }

            impl Cos for $float {
                type Output = Self;

                fn cos(&self) -> Self::Output {
                    (*self).cos()
                }
            }
        )+
    }
}

macro_rules! impl_float_sqrt {
    ( $( $float:ty )+ ) => {
        $(
            impl Sqrt for $float {
                type Output = Self;

                fn sqrt(&self) -> Self::Output {
                    (*self).sqrt()
                }
            }
        )+
    };
}

impl_nums_zero! { u8 u16 u32 u64 u128 usize }
impl_nums_zero! { i8 i16 i32 i64 i128 isize }
impl_nums_zero! { f32 f64 }

impl_nums_one! { u8 u16 u32 u64 u128 usize }
impl_nums_one! { i8 i16 i32 i64 i128 isize }
impl_nums_one! { f32 f64 }

impl_inums_abs! { i8 i16 i32 i64 i128 isize f32 f64 }
impl_unums_abs! { u8 u16 u32 u64 u128 usize }

impl_float_sin_cos! { f32 f64 }

// TODO: handle NaN
impl_float_sqrt! { f32 f64 }

impl<T> Scalar for T {}
