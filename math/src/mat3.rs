use std::ops::{Add, Div, Mul, Rem, Sub};

use crate::angle::Angle;
use crate::scalar::{One, Zero};
use crate::unit::Unit;
use crate::vector::Vec3;
use crate::{
    angle::AngleConst,
    matn::MatN,
    matrix::Owned,
    scalar::{Cos, Sin},
    storage::{ArrayStorage, Storage},
};

pub type Mat3<T, S = Owned<T, 3, 3>> = MatN<T, S, 3>;

impl<T> Mat3<T> {
    #[rustfmt::skip]
    pub fn new(
        c0r0: T, c1r0: T, c2r0: T,
        c0r1: T, c1r1: T, c2r1: T,
        c0r2: T, c1r2: T, c2r2: T,
    ) -> Self {
        Self::from_storage(ArrayStorage {
            data: [
                [c0r0, c0r1, c0r2],
                [c1r0, c1r1, c1r2],
                [c2r0, c2r1, c2r2],
            ],
        })
    }

    pub fn from_axis_angle<RS>(axis: &Unit<Vec3<T, RS>>, angle: Angle<T>) -> Self
    where
        T: Clone
            + AngleConst
            + Zero
            + One
            + PartialEq
            + Add<T, Output = T>
            + Sub<T, Output = T>
            + Mul<T, Output = T>
            + Div<T, Output = T>
            + Rem<T, Output = T>
            + Sin<Output = T>
            + Cos<Output = T>,
        RS: Storage<T, 3, 1>,
    {
        let rad = angle.to_rad_clamped();
        if rad == T::zero() {
            Self::identity()
        } else {
            let ux = axis.as_ref().x();
            let uy = axis.as_ref().y();
            let uz = axis.as_ref().z();
            let sqx = ux.clone() * ux.clone();
            let sqy = uy.clone() * uy.clone();
            let sqz = uz.clone() * uz.clone();
            let sin = rad.sin();
            let cos = rad.cos();
            let one_m_cos = T::zero() - cos.clone();

            Self::new(
                sqx.clone() + (T::one() - sqx.clone()) * cos.clone(),
                ux.clone() * uy.clone() * one_m_cos.clone() - uz.clone() * sin.clone(),
                ux.clone() * uz.clone() * one_m_cos.clone() + uy.clone() * sin.clone(),
                ux.clone() * uy.clone() * one_m_cos.clone() + uz.clone() * sin.clone(),
                sqy.clone() + (T::one() - sqy.clone()) * cos.clone(),
                uy.clone() * uz.clone() * one_m_cos.clone() - ux.clone() * sin.clone(),
                ux.clone() * uz.clone() * one_m_cos.clone() - uy.clone() * sin.clone(),
                uy.clone() * uz.clone() * one_m_cos + ux.clone() * sin,
                sqz.clone() + (T::one() - sqz) * cos,
            )
        }
    }
}
