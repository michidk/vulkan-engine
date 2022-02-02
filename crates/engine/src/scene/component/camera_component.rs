use std::rc::{Rc, Weak};

use engine_derive::InternalComponent;
use gfx_maths::{Mat4, Vec3};

use crate::scene::component::InfoBox;
use crate::scene::entity::Entity;

use super::Component;

#[derive(Debug, InternalComponent)]
pub struct CameraComponent {
    entity: Weak<Entity>,
    #[inspector(InfoBox{label: "Near: "})]
    near: f32,
    #[inspector(InfoBox{label: "Far: "})]
    far: f32,
    #[inspector(InfoBox{label: "Vertical FOV"})]
    fovy: f32,
}

impl Component for CameraComponent {
    fn create(entity: &Rc<Entity>) -> Rc<Self>
    where
        Self: Sized,
    {
        let res = Rc::new(Self {
            entity: Rc::downgrade(entity),
            near: 0.01,
            far: 1000.0,
            fovy: 60.0,
        });

        if let Some(scene) = entity.scene.upgrade() {
            scene.set_main_camera(Rc::downgrade(&res));
        }

        res
    }

    fn load(&self) {}

    fn start(&self) {}
}

#[repr(C)]
pub(crate) struct CameraUniformData {
    pub view_matrix: Mat4,
    pub projection_matrix: Mat4,
    pub inv_view_matrix: Mat4,
    pub inv_projection_matrix: Mat4,
    pub pos: Vec3,
}

impl CameraComponent {
    pub(crate) fn get_cam_data(&self, aspect: f32) -> CameraUniformData {
        let entity = self.entity.upgrade().unwrap();

        CameraUniformData {
            view_matrix: entity.get_view_matrix(),
            projection_matrix: Mat4::perspective_vulkan(
                self.fovy.to_radians(),
                self.near,
                self.far,
                aspect,
            ),
            inv_view_matrix: entity.get_inverse_view_matrix(),
            inv_projection_matrix: Mat4::inverse_perspective_vulkan(
                self.fovy.to_radians(),
                self.near,
                self.far,
                aspect,
            ),
            pos: entity.get_global_position(),
        }
    }
}
