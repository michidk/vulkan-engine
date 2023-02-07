use core::slice;
use std::fs;

use ve_shader_reflect::*;

#[test]
pub fn test_gpass_simple() {
    let root = env!("CARGO_MANIFEST_DIR");

    {
        let vert_spv = fs::read(format!("{root}/tests/shaders/gpass_simple_vert.spv")).unwrap();
        let info_vert = reflect_shader(unsafe {
            slice::from_raw_parts(vert_spv.as_ptr() as *const u32, vert_spv.len() / 4)
        })
        .expect("Failed to reflect vertex shader");

        assert!(info_vert.set_bindings.contains(&SetBinding {
            set: 0,
            binding: 0,
            var_name: String::from("u_FrameData"),
            data: SetBindingData::UniformBuffer {
                layout: BlockLayout {
                    block_name: String::from("FrameData"),
                    total_size: 256,
                    members: vec![
                        BlockMember {
                            kind: BlockMemberType::FloatMatrix(4),
                            offset: 0,
                            size: 64,
                            name: String::from("projMatrix"),
                        },
                        BlockMember {
                            kind: BlockMemberType::FloatMatrix(4),
                            offset: 64,
                            size: 64,
                            name: String::from("invProjMatrix"),
                        },
                        BlockMember {
                            kind: BlockMemberType::FloatMatrix(4),
                            offset: 128,
                            size: 64,
                            name: String::from("viewMatrix"),
                        },
                        BlockMember {
                            kind: BlockMemberType::FloatMatrix(4),
                            offset: 192,
                            size: 64,
                            name: String::from("invViewMatrix"),
                        },
                    ],
                }
            },
        }));
    }

    {
        let frag_spv = fs::read(format!("{root}/tests/shaders/gpass_simple_frag.spv")).unwrap();
        let info_frag = reflect_shader(unsafe {
            slice::from_raw_parts(frag_spv.as_ptr() as *const u32, frag_spv.len() / 4)
        })
        .expect("Failed to reflect fragment shader");

        assert!(info_frag.set_bindings.contains(&SetBinding {
            set: 1,
            binding: 1,
            var_name: String::from("u_AlbedoTex"),
            data: SetBindingData::SampledImage {
                dim: ImageDimension::Two,
            },
        }));

        assert!(info_frag.set_bindings.contains(&SetBinding {
            set: 1,
            binding: 0,
            var_name: String::from("u_Material"),
            data: SetBindingData::UniformBuffer {
                layout: BlockLayout {
                    block_name: String::from("MaterialData"),
                    total_size: 28,
                    members: vec![
                        BlockMember {
                            kind: BlockMemberType::Float,
                            offset: 0,
                            size: 4,
                            name: String::from("roughness"),
                        },
                        BlockMember {
                            kind: BlockMemberType::Float,
                            offset: 4,
                            size: 4,
                            name: String::from("metallic"),
                        },
                        BlockMember {
                            kind: BlockMemberType::FloatVector(3),
                            offset: 16,
                            size: 12,
                            name: String::from("tint"),
                        },
                    ],
                }
            },
        }));
    }
}
