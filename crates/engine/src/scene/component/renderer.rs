use std::{
    cell::RefCell,
    fmt,
    rc::{Rc, Weak},
};

use crate::scene::{entity::Entity, model::Model, transform::TransformData, Scene};

use super::Component;

pub struct RendererComponent {
    scene: RefCell<Weak<Scene>>,
    entity: RefCell<Weak<Entity>>,
    pub model: Rc<Model>,
}

impl RendererComponent {
    pub fn new(model: Rc<Model>) -> Rc<Self> {
        Rc::new(Self {
            scene: Weak::new().into(),
            entity: Weak::new().into(),
            model,
        })
    }
}

impl Component for RendererComponent {
    fn attach(&self, scene: Weak<Scene>, entity: Weak<Entity>) {
        *self.scene.borrow_mut() = scene;
        *self.entity.borrow_mut() = entity;
        // println!("Attach")
    }
    fn load(&self) {
        // println!("Load Ref");
    }
    fn start(&self) {
        // println!("Start");
    }
    fn update(&self, _delta: f32) {
        //     println!("Update");
    }

    fn render(&self, models: &mut Vec<(TransformData, Rc<Model>)>) {
        let entity = self.entity.borrow();
        if let Some(entity) = entity.upgrade() {
            let local2world = entity.get_local_to_world_matrix();
            let world2local = entity.get_world_to_local_matrix();
            let transform_data = TransformData {
                model_matrix: local2world,
                inv_model_matrix: world2local,
            };

            models.push((transform_data, self.model.clone()));
        }
    }
}

impl fmt::Debug for RendererComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RenderComponent").finish()
    }
}
