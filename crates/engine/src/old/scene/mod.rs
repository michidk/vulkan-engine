pub mod component;
pub mod entity;
pub mod light;
pub mod material;
pub mod model;
pub mod transform;

use std::cell::{Cell, RefCell};
use std::fmt::Debug;
use std::rc::{Rc, Weak};

use crate::old::core::input::Input;

use self::component::camera_component::CameraComponent;
use self::entity::Entity;
use self::light::Light;
use self::model::Model;
use self::transform::{Transform, TransformData};

pub struct Scene {
    pub(crate) root_entity: RefCell<Rc<Entity>>,
    pub(crate) main_camera: RefCell<Weak<CameraComponent>>,
    id_counter: Cell<u64>,
}

impl Scene {
    pub(crate) fn new() -> Rc<Self> {
        let mut root = Entity::new_root();
        let res = Rc::new(Self {
            root_entity: RefCell::new(Rc::new(Entity::new_root())),
            main_camera: RefCell::new(Weak::new()),
            id_counter: Cell::new(0),
        });
        root.scene = Rc::downgrade(&res);
        *res.root_entity.borrow_mut() = Rc::new(root);

        res
    }

    pub(crate) fn new_entity_id(&self) -> u64 {
        let res = self.id_counter.get();
        self.id_counter.set(res + 1);
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

    pub(crate) fn collect_renderables(&self) -> (Vec<(TransformData, Rc<Model>)>, Vec<Light>) {
        profile_function!();

        let mut models = Vec::new();
        let mut lights = Vec::new();

        self.root_entity
            .borrow()
            .collect_renderables(&mut models, &mut lights);

        (models, lights)
    }

    pub(crate) fn update(&self, input: &Input, delta: f32) {
        profile_function!();
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
