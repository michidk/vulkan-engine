use std::cell::{Cell, RefCell};
use std::fmt::Debug;
use std::rc::{Rc, Weak};

use gfx_maths::{Mat4, Quaternion, Vec3};

use super::component::Component;
use super::model::Model;
use super::transform::{Transform, TransformData};
use super::Scene;

#[derive(Debug)]
pub struct Entity {
    self_weak: Weak<Self>,
    parent: Weak<Entity>,
    pub name: String,
    transform: Transform,
    pub children: RefCell<Vec<Rc<Entity>>>,
    pub components: RefCell<Vec<Rc<dyn Component>>>,
    scene: RefCell<Weak<Scene>>,
    pub attached: Cell<bool>,
}

impl Entity {
    pub fn new(parent: Weak<Entity>, name: String) -> Rc<Entity> {
        Rc::new_cyclic(|self_weak| Entity {
            self_weak: self_weak.clone(),
            parent,
            name: name.to_string(),
            transform: Transform {
                position: Vec3::zero(),
                rotation: Quaternion::identity(),
                scale: Vec3::one(),
            },
            children: RefCell::new(Vec::new()),
            components: RefCell::new(Vec::new()),
            scene: RefCell::new(Weak::new()),
            attached: false.into(),
        })
    }

    pub fn new_with_transform(
        parent: Weak<Entity>,
        name: String,
        transform: Transform,
    ) -> Rc<Entity> {
        Rc::new_cyclic(|self_weak| Entity {
            self_weak: self_weak.clone(),
            parent,
            name: name.to_string(),
            transform,
            children: RefCell::new(Vec::new()),
            components: RefCell::new(Vec::new()),
            scene: RefCell::new(Weak::new()),
            attached: false.into(),
        })
    }

    pub(crate) fn new_root() -> Rc<Entity> {
        Rc::new_cyclic(|self_weak| Entity {
            self_weak: self_weak.clone(),
            parent: Weak::new(),
            name: "Scene Root".to_string(),
            transform: Transform {
                position: Vec3::zero(),
                rotation: Quaternion::identity(),
                scale: Vec3::one(),
            },
            children: RefCell::new(Vec::new()),
            components: RefCell::new(Vec::new()),
            scene: RefCell::new(Weak::new()),
            attached: false.into(),
        })
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
        self.parent.upgrade().is_none()
    }

    pub fn add_child(&self, child: Rc<Entity>) {
        self.children.borrow_mut().push(Rc::clone(&child));

        child.attach(Weak::clone(&self.scene.borrow()));
        // println!("attach child by add_child");
    }

    pub fn add_component(&self, component: Rc<dyn Component>) {
        self.components.borrow_mut().push(Rc::clone(&component));

        component.attach(
            Weak::clone(&self.scene.borrow()),
            Weak::clone(&self.self_weak),
        );
        // println!("attach comp by add_component");
    }

    pub fn attach(&self, scene: Weak<Scene>) {
        if self.attached.get() {
            return;
        }

        for comp in &*self.components.borrow() {
            comp.attach(Weak::clone(&scene), Weak::clone(&self.self_weak));
        }
        for child in &*self.children.borrow() {
            child.attach(Weak::clone(&scene));
        }

        *self.scene.borrow_mut() = scene;
        self.attached.set(true);
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
        self.transform.get_model_matrix()
    }

    pub fn get_local_to_world_matrix(&self) -> Mat4 {
        if let Some(parent) = self.parent.upgrade() {
            let parent_to_world = parent.get_local_to_world_matrix();
            parent_to_world * self.get_model_matrix()
        } else {
            self.get_model_matrix()
        }
    }

    pub fn get_inverse_model_matrix(&self) -> Mat4 {
        self.transform.get_inverse_model_matrix()
    }

    pub fn get_world_to_local_matrix(&self) -> Mat4 {
        if let Some(parent) = self.parent.upgrade() {
            let world_to_parent = parent.get_world_to_local_matrix();
            self.get_inverse_model_matrix() * world_to_parent
        } else {
            self.get_inverse_model_matrix()
        }
    }
}
