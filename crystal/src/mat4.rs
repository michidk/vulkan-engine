use crate::{
    angle::{Angle, AngleConst},
    scalar::{Cos, Sin},
    unit::Unit,
};

use std::ops::{Add, Div, Mul, MulAssign, Neg, Rem, Sub};

use crate::{
    matn::MatN,
    matrix::Matrix,
    scalar::{One, Zero},
    vector::Vec3,
};

pub type Mat4<T> = MatN<T, 4>;

impl<T> Mat4<T> {
    #[rustfmt::skip]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        c0r0: T, c1r0: T, c2r0: T, c3r0: T,
        c0r1: T, c1r1: T, c2r1: T, c3r1: T,
        c0r2: T, c1r2: T, c2r2: T, c3r2: T,
        c0r3: T, c1r3: T, c2r3: T, c3r3: T,
    ) -> Self {
        Self::from_data([
            [c0r0, c0r1, c0r2, c0r3],
            [c1r0, c1r1, c1r2, c1r3],
            [c2r0, c2r1, c2r2, c2r3],
            [c3r0, c3r1, c3r2, c3r3],
        ])
    }

    pub fn scale(factor: T) -> Self
    where
        T: Clone + Zero + One + Mul<T, Output = T>,
    {
        let mut matrix = &Self::identity() * factor;
        unsafe { *matrix.get_unchecked_mut((3, 3)) = T::one() };
        matrix
    }

    pub fn translate(direction: Vec3<T>) -> Self
    where
        T: Clone + Zero + One + Mul<T, Output = T>,
    {
        let mut matrix = Matrix::identity();
        unsafe {
            *matrix.get_unchecked_mut((0, 3)) = direction.get_unchecked((0, 0)).clone();
            *matrix.get_unchecked_mut((1, 3)) = direction.get_unchecked((1, 0)).clone();
            *matrix.get_unchecked_mut((2, 3)) = direction.get_unchecked((2, 0)).clone();
        };

        matrix
    }

    pub fn from_axis_angle(axis: &Unit<Vec3<T>>, angle: Angle<T>) -> Self
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
                sqx.clone() + (T::one() - sqx) * cos.clone(),
                ux.clone() * uy.clone() * one_m_cos.clone() - uz.clone() * sin.clone(),
                ux.clone() * uz.clone() * one_m_cos.clone() + uy.clone() * sin.clone(),
                T::zero(),
                //
                ux.clone() * uy.clone() * one_m_cos.clone() + uz.clone() * sin.clone(),
                sqy.clone() + (T::one() - sqy) * cos.clone(),
                uy.clone() * uz.clone() * one_m_cos.clone() - ux.clone() * sin.clone(),
                T::zero(),
                //
                ux.clone() * uz.clone() * one_m_cos.clone() - uy.clone() * sin.clone(),
                uy.clone() * uz.clone() * one_m_cos + ux.clone() * sin,
                sqz.clone() + (T::one() - sqz) * cos,
                T::zero(),
                //
                T::zero(),
                T::zero(),
                T::zero(),
                T::one(),
            )
        }
    }

    // This function only works for a matrix backed by an continues storage
    // like ArrayStorage as we cast the data pointer to a continues array of `T`.
    pub fn try_inverse(&self) -> Option<Self>
    where
        T: Copy
            + Zero
            + One
            + Add<T, Output = T>
            + Sub<T, Output = T>
            + Mul<T, Output = T>
            + MulAssign<T>
            + Div<T, Output = T>
            + Neg<Output = T>
            + PartialEq,
    {
        let mut out = Self::zero();

        unsafe {
            let m = *(self.data.as_ptr() as *const [T; 16]);

            *out.get_unchecked_mut((0, 0)) =
                m[5] * m[10] * m[15] - m[5] * m[11] * m[14] - m[9] * m[6] * m[15]
                    + m[9] * m[7] * m[14]
                    + m[13] * m[6] * m[11]
                    - m[13] * m[7] * m[10];

            *out.get_unchecked_mut((1, 0)) =
                -m[1] * m[10] * m[15] + m[1] * m[11] * m[14] + m[9] * m[2] * m[15]
                    - m[9] * m[3] * m[14]
                    - m[13] * m[2] * m[11]
                    + m[13] * m[3] * m[10];

            *out.get_unchecked_mut((2, 0)) =
                m[1] * m[6] * m[15] - m[1] * m[7] * m[14] - m[5] * m[2] * m[15]
                    + m[5] * m[3] * m[14]
                    + m[13] * m[2] * m[7]
                    - m[13] * m[3] * m[6];

            *out.get_unchecked_mut((3, 0)) =
                -m[1] * m[6] * m[11] + m[1] * m[7] * m[10] + m[5] * m[2] * m[11]
                    - m[5] * m[3] * m[10]
                    - m[9] * m[2] * m[7]
                    + m[9] * m[3] * m[6];

            *out.get_unchecked_mut((0, 1)) =
                -m[4] * m[10] * m[15] + m[4] * m[11] * m[14] + m[8] * m[6] * m[15]
                    - m[8] * m[7] * m[14]
                    - m[12] * m[6] * m[11]
                    + m[12] * m[7] * m[10];

            *out.get_unchecked_mut((1, 1)) =
                m[0] * m[10] * m[15] - m[0] * m[11] * m[14] - m[8] * m[2] * m[15]
                    + m[8] * m[3] * m[14]
                    + m[12] * m[2] * m[11]
                    - m[12] * m[3] * m[10];

            *out.get_unchecked_mut((2, 1)) =
                -m[0] * m[6] * m[15] + m[0] * m[7] * m[14] + m[4] * m[2] * m[15]
                    - m[4] * m[3] * m[14]
                    - m[12] * m[2] * m[7]
                    + m[12] * m[3] * m[6];

            *out.get_unchecked_mut((3, 1)) =
                m[0] * m[6] * m[11] - m[0] * m[7] * m[10] - m[4] * m[2] * m[11]
                    + m[4] * m[3] * m[10]
                    + m[8] * m[2] * m[7]
                    - m[8] * m[3] * m[6];

            *out.get_unchecked_mut((0, 2)) =
                m[4] * m[9] * m[15] - m[4] * m[11] * m[13] - m[8] * m[5] * m[15]
                    + m[8] * m[7] * m[13]
                    + m[12] * m[5] * m[11]
                    - m[12] * m[7] * m[9];

            *out.get_unchecked_mut((1, 2)) =
                -m[0] * m[9] * m[15] + m[0] * m[11] * m[13] + m[8] * m[1] * m[15]
                    - m[8] * m[3] * m[13]
                    - m[12] * m[1] * m[11]
                    + m[12] * m[3] * m[9];

            *out.get_unchecked_mut((2, 2)) =
                m[0] * m[5] * m[15] - m[0] * m[7] * m[13] - m[4] * m[1] * m[15]
                    + m[4] * m[3] * m[13]
                    + m[12] * m[1] * m[7]
                    - m[12] * m[3] * m[5];

            *out.get_unchecked_mut((0, 3)) =
                -m[4] * m[9] * m[14] + m[4] * m[10] * m[13] + m[8] * m[5] * m[14]
                    - m[8] * m[6] * m[13]
                    - m[12] * m[5] * m[10]
                    + m[12] * m[6] * m[9];

            *out.get_unchecked_mut((3, 2)) =
                -m[0] * m[5] * m[11] + m[0] * m[7] * m[9] + m[4] * m[1] * m[11]
                    - m[4] * m[3] * m[9]
                    - m[8] * m[1] * m[7]
                    + m[8] * m[3] * m[5];

            *out.get_unchecked_mut((1, 3)) =
                m[0] * m[9] * m[14] - m[0] * m[10] * m[13] - m[8] * m[1] * m[14]
                    + m[8] * m[2] * m[13]
                    + m[12] * m[1] * m[10]
                    - m[12] * m[2] * m[9];

            *out.get_unchecked_mut((2, 3)) =
                -m[0] * m[5] * m[14] + m[0] * m[6] * m[13] + m[4] * m[1] * m[14]
                    - m[4] * m[2] * m[13]
                    - m[12] * m[1] * m[6]
                    + m[12] * m[2] * m[5];

            *out.get_unchecked_mut((3, 3)) =
                m[0] * m[5] * m[10] - m[0] * m[6] * m[9] - m[4] * m[1] * m[10]
                    + m[4] * m[2] * m[9]
                    + m[8] * m[1] * m[6]
                    - m[8] * m[2] * m[5];

            let det = m[0] * *out.get_unchecked((0, 0))
                + m[1] * *out.get_unchecked((0, 1))
                + m[2] * *out.get_unchecked((0, 2))
                + m[3] * *out.get_unchecked((0, 3));

            if det != T::zero() {
                let inv_det = T::one() / det;

                for j in 0..4 {
                    for i in 0..4 {
                        *out.get_unchecked_mut((i, j)) *= inv_det;
                    }
                }
                Some(out)
            } else {
                None
            }
        }
    }
}

// TODO: make generic for Zero + Sin + Cos
impl Mat4<f32> {
    pub fn rotation_x(angle: Angle<f32>) -> Self {
        let rad = angle.to_rad_clamped();
        let sin = rad.sin();
        let cos = rad.cos();

        Self::from_data([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, cos, -sin, 0.0],
            [0.0, sin, cos, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn rotation_y(angle: Angle<f32>) -> Self {
        let rad = angle.to_rad_clamped();
        let sin = rad.sin();
        let cos = rad.cos();

        Self::from_data([
            [cos, 0.0, sin, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [-sin, 0.0, cos, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn rotation_z(angle: Angle<f32>) -> Self {
        let rad = angle.to_rad_clamped();
        let sin = rad.sin();
        let cos = rad.cos();

        Self::from_data([
            [cos, sin, 0.0, 0.0],
            [-sin, cos, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }
}

#[cfg(test)]
#[rustfmt::skip]
mod tests {
    use crate::test_util::MatrixCmp;

    use super::*;

    #[test]
    fn mat4_scaling() {
        let is = Mat4::scale(4.2);
        let should = Mat4::new(
            4.2, 0.0, 0.0, 0.0,
            0.0, 4.2, 0.0, 0.0,
            0.0, 0.0, 4.2, 0.0,
            0.0, 0.0, 0.0, 1.0,
        );

        MatrixCmp::<f32>::DEFAULT.eq(&is, &should);
    }

    #[test]
    fn mat4_translate() {
        let is = Mat4::translate(Vec3::new(0.2, 1.7, 7.0));
        let should = Mat4::new(
            1.0, 0.0, 0.0, 0.2,
            0.0, 1.0, 0.0, 1.7,
            0.0, 0.0, 1.0, 7.0,
            0.0, 0.0, 0.0, 1.0,
        );

        MatrixCmp::<f32>::DEFAULT.eq(&is, &should);
    }

    #[test]
    fn mat4_from_axis_angle() {
        let is = Mat4::from_axis_angle(&Unit::new_normalize(Vec3::new(2.0, 1.0, -33.2)), Angle::from_rad(2.0));
        let should = Mat4::new(
            -0.41103088395674775, -0.9046840553875477, -0.1122513802199109, 0.0,
            0.9098000079779422, -0.41486784839954377, 0.012190727938444015, 0.0,
            -0.057598245781191326, -0.09711554093899513, 0.9936050592620064, 0.0,
            0.0, 0.0, 0.0, 1.0
        );

        MatrixCmp::<f32>::DEFAULT.eq(&is, &should);
    }

    #[test]
    fn mat4_try_inverse() {
        let is = Mat4::scale(2.0);
        let should = Mat4::new(
            0.5, 0.0, 0.0, 0.0,
            0.0, 0.5, 0.0, 0.0,
            0.0, 0.0, 0.5, 0.0,
            0.0, 0.0, 0.0, 0.5,
        );

        MatrixCmp::<f32>::DEFAULT.eq(&is, &should);
    }
}
