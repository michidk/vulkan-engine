pub mod mesh;

use crate::{assets::mesh::Mesh, utils::color::Color};

pub struct Model {
    material: Material,
    mesh: Mesh,
}

pub struct Material {
    // shader: Shader,
}
