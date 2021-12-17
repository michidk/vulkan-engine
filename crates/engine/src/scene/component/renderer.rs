use std::{
    cell::RefCell,
    fmt,
    rc::{Rc, Weak},
};

use crate::scene::{entity::Entity, model::Model, Scene};

use super::Component;

pub struct RendererComponent {
    scene: Weak<Scene>,
    entity: Weak<RefCell<Entity>>,
    pub model: Rc<Model>,
}

impl RendererComponent {
    pub fn new(model: Rc<Model>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            scene: Weak::new(),
            entity: Weak::new(),
            model,
        }))
    }
}

impl Component for RendererComponent {
    fn attach(&mut self, scene: Weak<Scene>, entity: Weak<RefCell<Entity>>) {
        self.scene = scene;
        self.entity = entity;
        // println!("Attach")
    }
    fn load(&mut self) {
        if let Some(scene) = self.scene.upgrade() {
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
