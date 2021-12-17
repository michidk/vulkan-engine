pub mod component;
pub mod entity;
pub mod light;
pub mod material;
pub mod model;
pub mod transform;

use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::{Rc, Weak};

use self::entity::Entity;
use self::light::LightManager;
use self::model::Model;

pub struct Scene {
    self_weak: Weak<Scene>,
    pub light_manager: LightManager,
    pub models: RefCell<Vec<Rc<Model>>>,
    pub root_entity: Rc<RefCell<Entity>>,
}

impl Scene {
    pub(crate) fn new() -> Rc<Self> {
        let root = Entity::new_root();
        let self_rc = Rc::new_cyclic(|self_weak| Self {
            self_weak: self_weak.clone(),
            light_manager: LightManager::default(),
            models: RefCell::new(Vec::new()),
            root_entity: Rc::clone(&root),
        });
        root.borrow_mut().attach(Rc::downgrade(&self_rc));
        self_rc
    }

    pub fn add_model(&self, model: Rc<Model>) {
        self.models.borrow_mut().push(model);
    }

    pub fn add_entity(&self, entity: Rc<RefCell<Entity>>) {
        self.root_entity.borrow_mut().add_child(Rc::clone(&entity));

        entity.borrow_mut().attach(self.self_weak.clone());
    }

    pub fn load(&self) {
        self.root_entity.borrow().load();
    }
}

impl Drop for Scene {
    fn drop(&mut self) {
        println!("Dropping Scene");
    }
}

impl Debug for Scene {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Scene")
    }
}
