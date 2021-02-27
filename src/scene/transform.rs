use crystal::prelude::*;

pub struct Transform {
    position: Vec3<f32>,
    rotation: Quaternion<f32>,
    scale: Vec3<f32>,
}
