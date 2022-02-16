use std::{cell::RefCell, rc::Rc};

use super::input::Input;
use crate::old::scene::Scene;

pub(crate) struct GameLoop {
    input: Rc<RefCell<Input>>,
}

impl GameLoop {
    pub(crate) fn new(input: Rc<RefCell<Input>>) -> Self {
        Self { input }
    }

    pub(crate) fn init(&self) {}

    // TODO: implement Update, Render traits and then create type def of combined type; then have a list of them in SceneManager, and call update for all of them
    pub(crate) fn update(&self, scene: &Scene, delta: f32) {
        let input = self.input.borrow();
        scene.update(&input, delta);
    }
}
