mod builder;
mod meta;
mod parser;

use anyhow::{Context, Result};
use log::info;
use std::path::Path;
use ve_format::mesh::MeshData;

use crate::utils;

use self::meta::ObjMeta;

fn parse(path: &Path, meta: ObjMeta) -> Result<MeshData> {
    Ok(parser::parse(&path, meta)?.build_mesh()?)
}

fn serialize(mesh: MeshData) -> Result<Vec<u8>> {
    Ok(mesh.to_bytes().context("Could not serialize MeshData")?)
}

fn save(path: &Path, output_dir: &Path, data: Vec<u8>) -> Result<()> {
    let file_name = utils::file_name(path)?;
    let target = utils::combine_path(output_dir, file_name, "vem")?;
    utils::write_file(target, data)?;
    Ok(())
}

/// Parse meta from file called `file.meta` or alternativley from folder scoped meta file named `_.meta` or else use default meta
fn parse_meta(path: &Path) -> Result<ObjMeta> {
    let dir = path
        .parent()
        .with_context(|| format!("Path terminates in root or prefix: {}", path.display()))?;
    let meta_file = utils::file_name(&path)?;
    let path = utils::combine_path(dir, meta_file, "toml")?;

    let meta: ObjMeta;
    if path.exists() && path.is_file() {
        // load meta
        meta = ObjMeta::parse(&path)?;
    } else {
        // check if folder scoped meta exists
        let path = utils::combine_path(dir, "obj", "toml")?;
        if path.exists() && path.is_file() {
            // load meta
            meta = ObjMeta::parse(&path)?;
        } else {
            // create default meta
            meta = ObjMeta::default();
        }
    }

    Ok(meta)
}

pub(crate) fn process(path: &Path, output_dir: &Path) -> Result<()> {
    info!("Processing Wavefront `.obj`-file: `{}`", path.display());
    let meta = parse_meta(&path)?;
    save(path, output_dir, serialize(parse(path, meta)?)?)
}
