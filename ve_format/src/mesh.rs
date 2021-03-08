use crate::error::Result;
use crystal::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Vertex {
    pub position: Vec3<f32>,
    pub color: Vec3<f32>,
    pub normal: Vec3<f32>,
    pub uv: Vec2<f32>,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Submesh {
    pub faces: Vec<Face>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Face {
    pub indices: [u32; 3],
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MeshData {
    pub vertices: Vec<Vertex>,
    pub submeshes: Vec<Submesh>,
}

impl MeshData {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        Ok(bincode::deserialize::<MeshData>(&bytes)?)
    }

    pub fn from_file(path: &Path) -> Result<Self> {
        let data = std::fs::read(path)?;
        Ok(MeshData::from_bytes(data)?)
    }

    pub fn to_bytes(self) -> Result<Vec<u8>> {
        Ok(bincode::serialize(&self)?)
    }
}
