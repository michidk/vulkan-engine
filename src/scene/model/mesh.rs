use crystal::prelude::*;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Mesh {
    pub name: Option<String>,
    pub vertices: Vec<Vertex>,
    pub submeshes: Vec<Submesh>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Submesh {
    pub name: Option<String>,
    pub faces: Vec<Face>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Vertex {
    pub position: Vec3<f32>,
    pub color: Option<Vec3<f32>>,
    pub normal: Option<Vec3<f32>>,
    pub uv: Option<Vec2<f32>>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Face {
    pub indices: [usize; 3],
}
