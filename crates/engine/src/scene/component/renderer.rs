use std::{
    cell::RefCell,
    fmt,
    rc::{Rc, Weak},
};

use crate::scene::{entity::Entity, model::Model, transform::TransformData};

use super::Component;

pub struct RendererComponent {
    entity: RefCell<Weak<Entity>>,
    pub model: RefCell<Option<Rc<Model>>>,
}

impl Component for RendererComponent {
    fn create(entity: &Rc<Entity>) -> Rc<Self>
    where
        Self: Sized,
    {
        let res = RendererComponent {
            entity: Rc::downgrade(entity).into(),
            model: None.into(),
        };
        Rc::new(res)
    }

    fn load(&self) {
        // println!("Load Ref");
    }
    fn start(&self) {
        // println!("Start");
    }

    fn render(&self, models: &mut Vec<(TransformData, Rc<Model>)>) {
        let entity = self.entity.borrow();
        if let Some(entity) = entity.upgrade() {
            if let Some(model) = &*self.model.borrow() {
                let local2world = entity.get_local_to_world_matrix();
                let world2local = entity.get_world_to_local_matrix();
                let transform_data = TransformData {
                    model_matrix: local2world,
                    inv_model_matrix: world2local,
                };

                models.push((transform_data, model.clone()));
            }
        }
    }

    fn inspector_name(&self) -> &'static str {
        "RendererComponent"
    }
}

impl fmt::Debug for RendererComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RenderComponent").finish()
    }
}
