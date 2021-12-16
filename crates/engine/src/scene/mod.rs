pub mod light;
pub mod material;
pub mod model;
pub mod transform;
pub mod component;
pub mod entity;

use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::fmt::Debug;

use self::light::LightManager;
use self::model::Model;
use self::entity::Entity;

pub struct Scene {
    self_weak: Weak<RefCell<Scene>>,
    pub light_manager: LightManager,
    pub models: Vec<Rc<Model>>,
    pub root_entity: Rc<RefCell<Entity>>,
}

impl Scene {
    pub(crate) fn new() -> Rc<RefCell<Self>> {
        let root = Entity::new_root();
        let self_rc = Rc::new_cyclic(|self_weak| {

            RefCell::new(Self {
                self_weak: self_weak.clone(),
                light_manager: LightManager::default(),
                models: Vec::new(),
                root_entity: Rc::clone(&root),
            })

        });
        root.borrow_mut().attach(Rc::clone(&self_rc));
        self_rc
    }

    pub fn add_model(&mut self, model: Rc<Model>) {
        self.models.push(model);
    }

    pub fn add_entity(&self, entity: Rc<RefCell<Entity>>) {
        self.root_entity.borrow_mut().add_child(Rc::clone(&entity));

        if let Some(scene_rc) = self.self_weak.upgrade() {
            entity.borrow_mut().attach(Rc::clone(&scene_rc));
        } else {
            panic!("Cant add entity to scene without the scene having a valid self ref");
        }
    }

    pub fn load(&self) {
        self.root_entity.borrow().load();
    }
}

impl Debug for Scene {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Scene")
    }
}
