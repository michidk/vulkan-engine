use anyhow::{anyhow, Result};
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

pub fn file_name(path: &Path) -> Result<&str> {
    Ok(path
        .file_stem()
        .ok_or(anyhow!("No file stem found"))?
        .to_str()
        .ok_or(anyhow!("Can't convert file stem to string"))?)
}

pub fn combine_path(directory: &Path, file_name: &str, extension: &str) -> Result<PathBuf> {
    Ok(directory.join(format!("{}.{}", file_name, extension)))
}

pub fn write_file(target: PathBuf, data: Vec<u8>) -> Result<File> {
    let mut buffer = File::create(target)?;
    buffer.write(data.as_slice())?;
    Ok(buffer)
}