use std::rc::Rc;

use self::mesh::Mesh;

use super::{material::Material, transform::Transform};

pub mod mesh;

// TODO: model should be owned by the user not scene. then every rendercomponent has a ref (rc) to the model
pub struct Model {
    pub material: Rc<Material>,
    pub mesh: Rc<Mesh>,
    pub transform: Transform, // TODO: remove
}
