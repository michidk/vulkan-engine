use crate::math::Vec3;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Mesh {
    pub name: String,
    pub mtllib: String,
    pub vertices: Vec<Vertex>,
    pub faces: Vec<Face>,
    pub submeshes: Vec<Submesh>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Submesh {
    pub name: String,
    pub vertices: Vec<Vertex>,
    pub faces: Vec<Face>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    // uv: Vec3,
    // color: Vec3,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Face {
    pub indices: [u32; 3],
}
