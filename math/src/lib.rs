#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_imports)]
#![allow(unused_imports)]

mod angle;
mod mat3;
mod mat4;
mod matn;
mod matrix;
mod norm;
mod scalar;
mod storage;
mod unit;
mod vector;

pub mod prelude {
    pub use crate::angle::{Angle, IntoAngle};
    pub use crate::mat3::Mat3;
    pub use crate::mat4::Mat4;
    pub use crate::scalar::{One, Zero};
    pub use crate::unit::Unit;
    pub use crate::vector::{Vec2, Vec3, Vec4};
}
