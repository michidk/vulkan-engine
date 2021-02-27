pub mod material;
pub mod mesh;

use crate::{assets::mesh::Mesh, utils::color::Color};

use self::material::Material;

use super::transform::Transform;

pub struct Model {
    material: Material,
    mesh: Mesh,
    transform: Transform,
}
