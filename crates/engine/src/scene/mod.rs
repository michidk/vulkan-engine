pub mod component;
pub mod entity;
pub mod light;
pub mod material;
pub mod model;
pub mod transform;

use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::{Rc, Weak};

use crate::core::input::Input;

use self::component::camera_component::CameraComponent;
use self::entity::Entity;
use self::light::LightManager;
use self::model::Model;
use self::transform::{Transform, TransformData};

pub struct Scene {
    pub light_manager: LightManager,
    root_entity: RefCell<Rc<Entity>>,
    pub(crate) main_camera: RefCell<Weak<CameraComponent>>,
}

impl Scene {
    pub(crate) fn new() -> Rc<Self> {
        let mut root = Entity::new_root();
        let res = Rc::new(Self {
            light_manager: LightManager::default(),
            root_entity: RefCell::new(Rc::new(Entity::new_root())),
            main_camera: RefCell::new(Weak::new()),
        });
        root.scene = Rc::downgrade(&res);
        *res.root_entity.borrow_mut() = Rc::new(root);

        res
    }

    pub fn new_entity(self: &Rc<Self>, name: String) -> Rc<Entity> {
        Entity::new(self, &*self.root_entity.borrow(), name)
    }

    pub fn new_entity_with_transform(
        self: &Rc<Self>,
        name: String,
        transform: Transform,
    ) -> Rc<Entity> {
        Entity::new_with_transform(self, &*self.root_entity.borrow(), name, transform)
    }

    pub fn load(&self) {
        self.root_entity.borrow().load();
    }

    pub(crate) fn collect_renderables(&self) -> Vec<(TransformData, Rc<Model>)> {
        let mut res = Vec::new();

        self.root_entity.borrow().collect_renderables(&mut res);

        res
    }

    pub(crate) fn update(&self, input: &Input, delta: f32) {
        self.root_entity.borrow().update(input, delta);
    }

    pub(crate) fn set_main_camera(&self, cam: Weak<CameraComponent>) {
        *self.main_camera.borrow_mut() = cam;
    }
}

impl Debug for Scene {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Scene")
    }
}
