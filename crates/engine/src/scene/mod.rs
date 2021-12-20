pub mod component;
pub mod entity;
pub mod light;
pub mod material;
pub mod model;
pub mod transform;

use std::fmt::Debug;
use std::rc::Rc;

use self::entity::Entity;
use self::light::LightManager;
use self::model::Model;
use self::transform::{Transform, TransformData};

pub struct Scene {
    pub light_manager: LightManager,
    pub root_entity: Rc<Entity>,
}

impl Scene {
    pub(crate) fn new() -> Rc<Self> {
        Rc::new_cyclic(|self_weak| {
            let mut root = Entity::new_root();
            root.scene = self_weak.clone();
            Self {
                light_manager: LightManager::default(),
                root_entity: Rc::new(root),
            }
        })
    }

    pub fn new_entity(self: &Rc<Self>, name: String) -> Rc<Entity> {
        Entity::new(self, name)
    }

    pub fn new_entity_with_transform(
        self: &Rc<Self>,
        name: String,
        transform: Transform,
    ) -> Rc<Entity> {
        Entity::new_with_transform(self, name, transform)
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
