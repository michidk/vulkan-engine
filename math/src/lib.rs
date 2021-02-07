#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_imports)]
#![allow(unused_imports)]

mod angle;
mod mat4;
mod matn;
mod matrix;
mod scalar;
mod storage;
mod vector;

pub mod prelude {
    pub use crate::angle::{Angle, ToAngle};
    pub use crate::mat4::Mat4;
    pub use crate::vector::{Vec2, Vec3, Vec4};
}
