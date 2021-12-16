use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::{Rc, Weak};

use gfx_maths::Mat4;

use super::component::Component;
use super::Scene;

#[derive(Debug)]
pub struct Entity {
    self_weak: Weak<RefCell<Self>>,
    parent: Option<Rc<RefCell<Entity>>>,
    pub name: String,
    transform: Mat4,
    pub children: RefCell<Vec<Rc<RefCell<Entity>>>>,
    pub components: RefCell<Vec<Rc<RefCell<dyn Component>>>>,
    scene: Option<Rc<RefCell<Scene>>>,
    pub attached: bool,
}

impl Entity {
    pub fn new(parent: Rc<RefCell<Entity>>, name: String) -> Rc<RefCell<Entity>> {
        Rc::new_cyclic(|self_weak| {
            RefCell::new(Entity {
                self_weak: self_weak.clone(),
                parent: Some(parent),
                name: name.to_string(),
                transform: Mat4::identity(),
                children: RefCell::new(Vec::new()),
                components: RefCell::new(Vec::new()),
                scene: None,
                attached: false,
            })
        })
    }

    pub(crate) fn new_root() -> Rc<RefCell<Entity>> {
        Rc::new_cyclic(|self_weak| {
            RefCell::new(Entity {
                self_weak: self_weak.clone(),
                parent: None,
                name: "Scene Root".to_string(),
                transform: Mat4::identity(),
                children: RefCell::new(Vec::new()),
                components: RefCell::new(Vec::new()),
                scene: None,
                attached: false,
            })
        })
    }

    pub fn load(&self) {
        self.components.borrow().iter().for_each(|component| {
            component.borrow_mut().load();
        });
        self.children.borrow().iter().for_each(|child| {
            child.borrow_mut().load();
        });
    }

    pub fn is_root(&self) -> bool {
        !self.parent.is_some()
    }

    pub fn add_child(&self, child: Rc<RefCell<Entity>>) {
        self.children.borrow_mut().push(Rc::clone(&child));

        if let Some(scene) = &self.scene {
            child.borrow_mut().attach(Rc::clone(scene));
            // println!("attach child by add_child");
        }
    }

    pub fn add_component(&self, component: Rc<RefCell<dyn Component>>) {
        let comp = Rc::clone(&component);
        self.components.borrow_mut().push(Rc::clone(&component));

        if let Some(scene) = &self.scene {
            comp.borrow_mut()
                .attach(Rc::clone(&scene), Weak::clone(&self.self_weak));
            // println!("attach comp by add_component");
        }
    }

    pub fn attach(&mut self, scene: Rc<RefCell<Scene>>) {
        if self.attached {
            return;
        }

        for comp in &*self.components.borrow() {
            comp.borrow_mut()
                .attach(Rc::clone(&scene), Weak::clone(&self.self_weak));
        }
        for child in &*self.children.borrow() {
            child.borrow_mut().attach(Rc::clone(&scene));
        }

        self.scene = Some(scene);
        self.attached = true;
        println!("attach by attach()");
    }
}
