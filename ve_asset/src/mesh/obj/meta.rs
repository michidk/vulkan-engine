use anyhow::Result;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Default, Clone, Copy, Deserialize)]
pub(crate) struct ObjMeta {
    flip_normals: [bool; 3],
    calculate_normals: bool,
}

impl ObjMeta {
    pub(crate) fn parse(path: &Path) -> Result<Self> {
        let data = std::fs::read(path)?;
        let meta: Self = toml::from_slice(&data)?;
        Ok(meta)
    }
}
