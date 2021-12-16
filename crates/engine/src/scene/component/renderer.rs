use std::{
    cell::RefCell,
    fmt,
    rc::{Rc, Weak},
};

use crate::scene::{entity::Entity, model::Model, Scene};

use super::Component;

pub struct RendererComponent {
    scene: Option<Rc<RefCell<Scene>>>,
    entity: Option<Rc<RefCell<Entity>>>,
    pub model: Rc<Model>,
}

impl RendererComponent {
    pub fn new(model: Rc<Model>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            scene: None,
            entity: None,
            model: model,
        }))
    }
}

impl Component for RendererComponent {
    fn attach(&mut self, scene: Rc<RefCell<Scene>>, entity: Weak<RefCell<Entity>>) {
        self.scene = Some(scene);
        self.entity = entity.upgrade();
        // println!("Attach")
    }
    fn load(&mut self) {
        if let Some(scene) = &self.scene {
            scene.borrow().add_model(Rc::clone(&self.model));
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
