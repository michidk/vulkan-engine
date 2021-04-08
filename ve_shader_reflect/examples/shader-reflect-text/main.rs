use std::{fs, slice};

use ve_shader_reflect::*;

fn main() {
    let path = format!(
        "{}/examples/shaders/gpass_simple_frag.spv",
        env!("CARGO_MANIFEST_DIR")
    );
    let spv = fs::read(path).unwrap();
    let info = ve_shader_reflect::reflect_shader(unsafe {
        slice::from_raw_parts(spv.as_ptr() as *const u32, spv.len() / 4)
    })
    .expect("Failed to reflect shader");

    print_bindings(&info.set_bindings);
}

fn print_bindings(bindings: &[SetBinding]) {
    for b in bindings {
        print!("layout (set={}, binding={}) uniform ", b.set, b.binding);

        match &b.data {
            SetBindingData::Sampler => {
                print!("sampler ");
            }
            SetBindingData::Image { dim } => match dim {
                ImageDimension::One => print!("image1D "),
                ImageDimension::Two => print!("image2D "),
                ImageDimension::Three => print!("image3D "),
                ImageDimension::Cube => print!("imageCube "),
                ImageDimension::SubpassInput => print!("subpassInput "),
            },
            SetBindingData::SampledImage { dim } => match dim {
                ImageDimension::One => print!("sampler1D "),
                ImageDimension::Two => print!("sampler2D "),
                ImageDimension::Three => print!("sampler3D "),
                ImageDimension::Cube => print!("samplerCube "),
                ImageDimension::SubpassInput => print!("samplerSubpassInput "),
            },
            SetBindingData::UniformBuffer { layout } => {
                print!("{} {{\n", layout.block_name);
                print_block_layout(layout);
                print!("}} ");
            }
        }

        println!("{};", b.var_name);
    }
}

fn print_block_layout(layout: &BlockLayout) {
    for member in &layout.members {
        print!("\tlayout (Offset={}) ", member.offset);

        match member.kind {
            BlockMemberType::Unsupported => print!("<unsupported> "),
            BlockMemberType::Float => print!("float "),
            BlockMemberType::FloatVector(dim) => print!("vec{} ", dim),
            BlockMemberType::FloatMatrix(dim) => print!("mat{} ", dim),
        }

        println!("{}; // Size={}", member.name, member.size);
    }
}
