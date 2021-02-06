use crate::math::Vec3;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Mesh {
    pub name: Option<String>,
    pub submeshes: Vec<Submesh>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Submesh {
    pub name: Option<String>,
    pub vertices: Vec<Vertex>,
    pub faces: Vec<Face>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vertex {
    pub position: Vec3,
    pub color: Option<Vec3>,
    pub normal: Option<Vec3>,
    pub uv: Option<Vec3>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Face {
    pub indices: [usize; 3],
}
