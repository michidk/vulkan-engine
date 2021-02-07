pub trait Zero {
    fn zero() -> Self;
}

pub trait One {
    fn one() -> Self;
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

impl_nums_zero! { u8 u16 u32 u64 u128 usize }
impl_nums_zero! { i8 i16 i32 i64 i128 isize }
impl_nums_zero! { f32 f64 }

impl_nums_one! { u8 u16 u32 u64 u128 usize }
impl_nums_one! { i8 i16 i32 i64 i128 isize }
impl_nums_one! { f32 f64 }

impl<T> Scalar for T {}
