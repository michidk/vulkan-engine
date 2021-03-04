use crystal::prelude::*;

pub struct Transform {
    pub position: Vec3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vec3<f32>,
}

impl Transform {
    pub fn get_transform_data(&self) -> TransformData {
        let model_matrix = &(&Mat4::translate(self.position) * &Mat4::from(self.rotation)) * &Mat4::scale(self.scale);
        let inv_model_matrix = 
            &(&Mat4::scale(Vec3::new(1.0 / self.scale.x(), 1.0 / self.scale.y(), 1.0 / self.scale.z())) * 
            &Mat4::from(self.rotation.conjugated())) *
            &Mat4::translate(-self.position);
        
            TransformData {
                model_matrix,
                inv_model_matrix
            }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TransformData {
    pub model_matrix: Mat4<f32>,
    pub inv_model_matrix: Mat4<f32>,
}
