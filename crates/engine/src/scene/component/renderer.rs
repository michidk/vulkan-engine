use std::{
    cell::RefCell,
    fmt,
    rc::{Rc, Weak},
};

use crate::scene::{entity::Entity, model::Model, Scene};

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
        if let Some(scene) = self.scene.borrow().upgrade() {
            scene.add_model(Rc::clone(&self.model));
        }
        // println!("Load Ref");
    }
    fn start(&self) {
        // println!("Start");
    }
    fn update(&self) {
        //     println!("Update");
    }
}

impl fmt::Debug for RendererComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RenderComponent").finish()
    }
}
