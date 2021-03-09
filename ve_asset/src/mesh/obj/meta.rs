use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Copy, Deserialize)]
pub(crate) struct ObjMeta {
    pub(crate) flip_normals: [bool; 3],
    pub(crate) calculate_normals: bool,
}

impl Default for ObjMeta {
    fn default() -> Self {
        Self {
            flip_normals: [true, false, false],
            calculate_normals: false
        }
    }
}

impl ObjMeta {
    pub(crate) fn parse(path: &Path) -> Result<Self> {
        let data = std::fs::read(path)
            .with_context(|| format!("Could not read file: {}", path.display()))?;
        let meta: Self = toml::from_slice(&data)
            .with_context(|| format!("Could not parse meta file: {}", path.display()))?;
        Ok(meta)
    }
}
