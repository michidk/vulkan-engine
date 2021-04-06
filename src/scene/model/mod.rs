use std::rc::Rc;

use self::mesh::Mesh;

use super::{material::MaterialInterface, transform::Transform};

pub mod mesh;

pub struct Model {
    pub material: Rc<dyn MaterialInterface>,
    pub mesh: Rc<Mesh>,
    pub transform: Transform,
}
