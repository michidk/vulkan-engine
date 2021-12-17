use anyhow::{anyhow, Context, Result};
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

pub fn file_name(path: &Path) -> Result<&str> {
    path.file_stem()
        .ok_or_else(|| anyhow!("No file stem found"))?
        .to_str()
        .ok_or_else(|| anyhow!("Can't convert file stem to string"))
}

pub fn combine_path(directory: &Path, file_name: &str, extension: &str) -> Result<PathBuf> {
    Ok(directory.join(format!("{}.{}", file_name, extension)))
}

pub fn write_file(target: PathBuf, data: Vec<u8>) -> Result<File> {
    let mut buffer = File::create(&target)
        .with_context(|| format!("Could not create file: {}", &target.display()))?;
    buffer
        .write(data.as_slice())
        .with_context(|| format!("Could not write data to file: {}", &target.display()))?;
    Ok(buffer)
}
