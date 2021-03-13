#![allow(dead_code)]
#![allow(unused_imports)]
#![feature(array_methods)]

mod angle;
mod index;
mod mat3;
mod mat4;
mod matn;
mod matrix;
mod norm;
mod quaternion;
mod scalar;
mod unit;
mod vector;

#[cfg(test)]
mod test_util;

pub mod prelude {
    pub use crate::angle::{Angle, IntoAngle};
    pub use crate::mat3::Mat3;
    pub use crate::mat4::Mat4;
    pub use crate::quaternion::Quaternion;
    pub use crate::scalar::{One, Zero, Cross};
    pub use crate::unit::Unit;
    pub use crate::vector::{Vec2, Vec3, Vec4};
}
