use gfx_maths::*;

#[derive(Debug)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quaternion,
    pub scale: Vec3,
}

impl Transform {
    pub fn get_model_matrix(&self) -> Mat4 {
        Mat4::local_to_world(self.position, self.rotation, self.scale)
    }

    pub fn get_inverse_model_matrix(&self) -> Mat4 {
        Mat4::world_to_local(self.position, self.rotation, self.scale)
    }

    pub fn get_transform_data(&self) -> TransformData {
        TransformData {
            model_matrix: self.get_model_matrix(),
            inv_model_matrix: self.get_inverse_model_matrix(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TransformData {
    pub model_matrix: Mat4,
    pub inv_model_matrix: Mat4,
}
