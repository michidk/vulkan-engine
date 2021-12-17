pub mod component;
pub mod entity;
pub mod light;
pub mod material;
pub mod model;
pub mod transform;

use std::fmt::Debug;
use std::rc::{Rc, Weak};

use self::entity::Entity;
use self::light::LightManager;
use self::model::Model;
use self::transform::TransformData;

pub struct Scene {
    self_weak: Weak<Scene>,
    pub light_manager: LightManager,
    pub root_entity: Rc<Entity>,
}

impl Scene {
    pub(crate) fn new() -> Rc<Self> {
        let root = Entity::new_root();
        let self_rc = Rc::new_cyclic(|self_weak| Self {
            self_weak: self_weak.clone(),
            light_manager: LightManager::default(),
            root_entity: Rc::clone(&root),
        });
        root.attach(Rc::downgrade(&self_rc));
        self_rc
    }

    pub fn add_entity(&self, entity: Rc<Entity>) {
        self.root_entity.add_child(Rc::clone(&entity));

        entity.attach(self.self_weak.clone());
    }

    pub fn load(&self) {
        self.root_entity.load();
    }

    pub fn collect_renderables(&self) -> Vec<(TransformData, Rc<Model>)> {
        let mut res = Vec::new();

        self.root_entity.collect_renderables(&mut res);

        res
    }

    pub fn update(&self, delta: f32) {
        self.root_entity.update(delta);
    }
}

impl Debug for Scene {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Scene")
    }
}
