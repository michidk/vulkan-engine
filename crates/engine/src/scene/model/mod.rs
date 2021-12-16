use std::rc::Rc;

use self::mesh::Mesh;

use super::{material::Material, transform::Transform};

pub mod mesh;

pub struct Model {
    pub material: Rc<Material>,
    pub mesh: Rc<Mesh>,
    pub transform: Transform, // TODO: remove
}
