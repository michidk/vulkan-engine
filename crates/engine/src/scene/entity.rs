use std::cell::{Cell, RefCell};
use std::fmt::Debug;
use std::rc::{Rc, Weak};

use gfx_maths::Mat4;

use super::component::Component;
use super::Scene;

#[derive(Debug)]
pub struct Entity {
    self_weak: Weak<Self>,
    parent: Weak<Entity>,
    pub name: String,
    transform: Mat4,
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
            transform: Mat4::identity(),
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
            transform: Mat4::identity(),
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
}
