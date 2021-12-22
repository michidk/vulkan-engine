use gfx_maths::*;

#[derive(Debug)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quaternion,
    pub scale: Vec3,
}

impl Transform {
    pub(crate) fn get_model_matrix(&self) -> Mat4 {
        Mat4::local_to_world(self.position, self.rotation, self.scale)
    }

    pub(crate) fn get_inverse_model_matrix(&self) -> Mat4 {
        Mat4::world_to_local(self.position, self.rotation, self.scale)
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TransformData {
    pub model_matrix: Mat4,
    pub inv_model_matrix: Mat4,
}
