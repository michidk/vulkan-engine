use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::{Rc, Weak};

use gfx_maths::{Mat4, Quaternion, Vec3};

use super::component::Component;
use super::model::Model;
use super::transform::{Transform, TransformData};
use super::Scene;

#[derive(Debug)]
pub struct Entity {
    parent: RefCell<Weak<Entity>>,
    pub name: String,
    pub transform: RefCell<Transform>,
    pub children: RefCell<Vec<Rc<Entity>>>,
    pub components: RefCell<Vec<Rc<dyn Component>>>,
    pub scene: Weak<Scene>,
}

impl Entity {
    pub(crate) fn new(scene: &Rc<Scene>, parent: &Rc<Entity>, name: String) -> Rc<Entity> {
        let res = Rc::new(Entity {
            parent: Rc::downgrade(parent).into(),
            name,
            transform: Transform {
                position: Vec3::zero(),
                rotation: Quaternion::identity(),
                scale: Vec3::one(),
            }
            .into(),
            children: RefCell::new(Vec::new()),
            components: RefCell::new(Vec::new()),
            scene: Rc::downgrade(scene),
        });

        parent.add_child(res.clone());

        res
    }

    pub(crate) fn new_with_transform(
        scene: &Rc<Scene>,
        parent: &Rc<Entity>,
        name: String,
        transform: Transform,
    ) -> Rc<Entity> {
        let res = Rc::new(Entity {
            parent: Rc::downgrade(parent).into(),
            name,
            transform: transform.into(),
            children: RefCell::new(Vec::new()),
            components: RefCell::new(Vec::new()),
            scene: Rc::downgrade(scene),
        });

        parent.add_child(res.clone());

        res
    }

    pub(crate) fn new_root() -> Entity {
        Entity {
            parent: Weak::new().into(),
            name: "Scene Root".to_string(),
            transform: Transform {
                position: Vec3::zero(),
                rotation: Quaternion::identity(),
                scale: Vec3::one(),
            }
            .into(),
            children: RefCell::new(Vec::new()),
            components: RefCell::new(Vec::new()),
            scene: Weak::new(),
        }
    }

    pub fn load(&self) {
        // println!("Loading Entity: {}", self.name);
        self.components.borrow().iter().for_each(|component| {
            component.load();
        });
        self.children.borrow().iter().for_each(|child| {
            child.load();
        });
    }

    pub fn is_root(&self) -> bool {
        self.parent.borrow().upgrade().is_none()
    }

    fn add_child(&self, child: Rc<Entity>) {
        self.children.borrow_mut().push(child);
    }

    pub fn attach_to(self: &Rc<Entity>, new_parent: &Rc<Entity>) {
        *self.parent.borrow_mut() = Rc::downgrade(new_parent);
        new_parent.add_child(self.clone());
    }

    pub fn new_component<T: 'static + Component>(self: &Rc<Self>) -> Rc<T> {
        let comp = T::create(self);

        self.components.borrow_mut().push(comp.clone());

        comp
    }

    pub fn collect_renderables(&self, models: &mut Vec<(TransformData, Rc<Model>)>) {
        for comp in &*self.components.borrow() {
            comp.render(models);
        }

        for child in &*self.children.borrow() {
            child.collect_renderables(models);
        }
    }

    pub fn get_model_matrix(&self) -> Mat4 {
        self.transform.borrow().get_model_matrix()
    }

    pub fn get_local_to_world_matrix(&self) -> Mat4 {
        if let Some(parent) = self.parent.borrow().upgrade() {
            let parent_to_world = parent.get_local_to_world_matrix();
            parent_to_world * self.get_model_matrix()
        } else {
            self.get_model_matrix()
        }
    }

    pub fn get_inverse_model_matrix(&self) -> Mat4 {
        self.transform.borrow().get_inverse_model_matrix()
    }

    pub fn get_world_to_local_matrix(&self) -> Mat4 {
        if let Some(parent) = self.parent.borrow().upgrade() {
            let world_to_parent = parent.get_world_to_local_matrix();
            self.get_inverse_model_matrix() * world_to_parent
        } else {
            self.get_inverse_model_matrix()
        }
    }

    pub fn update(&self, delta: f32) {
        for comp in &*self.components.borrow() {
            comp.update(delta);
        }

        for child in &*self.children.borrow() {
            child.update(delta);
        }
    }
}
