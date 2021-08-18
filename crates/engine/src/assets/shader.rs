use std::{
    convert::TryInto,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use ash::vk;

const FOLDER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/shaders");

pub enum ShaderKind {
    Vertex,
    Fragment,
}

impl ShaderKind {
    fn extension(&self) -> &str {
        use ShaderKind::*;
        match self {
            Vertex => "vert",
            Fragment => "frag",
        }
    }
}

pub fn load(name: &str, kind: ShaderKind, code_ref: &mut Vec<u32>) -> vk::ShaderModuleCreateInfo {
    let folder = Path::new(FOLDER);
    let filename = format!("{}-{}.spv", name, kind.extension());
    let file = folder.join(&filename);

    get_file_as_bytes(file, code_ref);

    *vk::ShaderModuleCreateInfo::builder().code(&code_ref)
}

fn get_file_as_bytes(file: PathBuf, dst: &mut Vec<u32>) {
    let len = file
        .metadata()
        .unwrap_or_else(|_| panic!("Couldn't read file metadata of shader {}", file.display()))
        .len()
        .try_into()
        .unwrap_or_else(|_| {
            panic!(
                "Couldn't read file size into a usize of shader {}",
                file.display()
            )
        });

    let u32_len = std::mem::size_of::<u32>();
    assert!(len % u32_len == 0, "Parsed shader file wrong length.");

    *dst = vec![0u32; len / u32_len]; // overwrite array stored in pointer with the one we actually need

    let buf = unsafe {
        std::slice::from_raw_parts_mut(dst.as_mut_ptr() as *mut u8, len) // assumes little endian, convert later if neccessary
    };

    let mut f = File::open(&file)
        .unwrap_or_else(|_| panic!("Couldn't read shader file {}", file.display()));
    f.read_exact(buf)
        .unwrap_or_else(|_| panic!("Couldn't read shader into the buffer {}", file.display()));

    // convert to big endian, if the host system uses big endian
    if cfg!(target_endian = "big") {
        for n in dst {
            *n = n.to_le();
        }
    }
}
