use std::rc::{Rc, Weak};

use egui::TextEdit;
use gfx_maths::{Mat4, Vec3};

use crate::scene::entity::Entity;

use super::Component;

#[derive(Debug)]
pub struct CameraComponent {
    entity: Weak<Entity>,
    near: f32,
    far: f32,
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

    fn inspector_name(&self) -> &'static str {
        "CameraComponent"
    }

    fn render_inspector(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Near");
            let mut text = self.near.to_string();
            ui.add_enabled(false, TextEdit::singleline(&mut text));
        });

        ui.horizontal(|ui| {
            ui.label("Far");
            let mut text = self.far.to_string();
            ui.add_enabled(false, TextEdit::singleline(&mut text));
        });

        ui.horizontal(|ui| {
            ui.label("Vertical FOV");
            let mut text = self.fovy.to_string();
            ui.add_enabled(false, TextEdit::singleline(&mut text));
        });
    }
}

#[repr(C)]
pub(crate) struct CamData {
    pub view_matrix: Mat4,
    pub projection_matrix: Mat4,
    pub inv_view_matrix: Mat4,
    pub inv_projection_matrix: Mat4,
    pub pos: Vec3,
}

impl CameraComponent {
    pub(crate) fn get_cam_data(&self, aspect: f32) -> CamData {
        let entity = self.entity.upgrade().unwrap();

        CamData {
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
