mod builder;
mod meta;
mod parser;

use anyhow::Result;
use std::path::Path;
use ve_format::mesh::MeshData;

use crate::utils;

fn parse(path: &Path) -> Result<MeshData> {
    Ok(parser::parse(&path)?.build_mesh()?)
}

fn serialize(mesh: MeshData) -> Result<Vec<u8>> {
    Ok(mesh.to_bytes()?)
}

fn save(path: &Path, output_dir: &Path, data: Vec<u8>) -> Result<()> {
    let file_name = utils::file_name(path)?;
    let target = utils::combine_path(output_dir, file_name, "vem")?;
    utils::write_file(target, data)?;
    Ok(())
}

pub(crate) fn process(path: &Path, output_dir: &Path) -> Result<()> {
    save(path, output_dir, serialize(parse(path)?)?)
}
