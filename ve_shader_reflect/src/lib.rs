use spirv_cross::{
    glsl,
    spirv::{Ast, Decoration, Module, Resource, Type},
};

pub type Error = spirv_cross::ErrorCode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockMemberType {
    Unsupported,
    Float,
    FloatVector(u32),
    FloatMatrix(u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockMember {
    pub kind: BlockMemberType,
    pub offset: u32,
    pub size: u32,
    pub name: String,
}

impl BlockMember {
    fn equal_ignore_names(&self, r: &BlockMember) -> bool {
        self.kind == r.kind && self.offset == r.offset && self.size == r.size
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockLayout {
    pub members: Vec<BlockMember>,
    pub block_name: String,
    pub total_size: u32,
}

impl BlockLayout {
    fn equal_ignore_names(&self, r: &BlockLayout) -> bool {
        if self.total_size != r.total_size {
            false
        } else {
            for (a, b) in self.members.iter().zip(r.members.iter()) {
                if !a.equal_ignore_names(b) {
                    return false;
                }
            }
            true
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageDimension {
    One,
    Two,
    Three,
    Cube,
    SubpassInput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetBindingData {
    Sampler,
    Image { dim: ImageDimension },
    SampledImage { dim: ImageDimension },
    UniformBuffer { layout: BlockLayout },
}

impl SetBindingData {
    fn equal_ignore_names(&self, r: &SetBindingData) -> bool {
        match self {
            SetBindingData::UniformBuffer { layout } => match r {
                SetBindingData::UniformBuffer { layout: r_layout } => {
                    layout.equal_ignore_names(r_layout)
                }
                _ => false,
            },
            _ => self == r,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetBinding {
    pub set: u32,
    pub binding: u32,
    pub data: SetBindingData,
    pub var_name: String,
}

impl SetBinding {
    fn equal_ignore_names(&self, r: &SetBinding) -> bool {
        self.set == r.set && self.binding == r.binding && self.data.equal_ignore_names(&r.data)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderInfo {
    pub set_bindings: Vec<SetBinding>,
    pub push_constants: Option<BlockLayout>,
}

type Refl = Ast<glsl::Target>;
type Result<T> = std::result::Result<T, Error>;

pub fn reflect_shader(spv_code: &[u32]) -> Result<ShaderInfo> {
    let module = Module::from_words(spv_code);
    let mut refl = Refl::parse(&module)?;

    let set_bindings = reflect_shader_bindings(&mut refl)?;

    Ok(ShaderInfo {
        set_bindings,
        push_constants: None,
    })
}

fn reflect_shader_bindings(refl: &mut Refl) -> Result<Vec<SetBinding>> {
    let ubos = refl.get_shader_resources()?.uniform_buffers;

    let mut set_bindings = Vec::with_capacity(ubos.len());
    reflect_ubos(refl, &ubos, &mut set_bindings)?;
    reflect_image_bindings(refl, &mut set_bindings)?;

    Ok(set_bindings)
}

fn reflect_image_bindings(refl: &mut Refl, set_bindings: &mut Vec<SetBinding>) -> Result<()> {
    let resources = refl.get_shader_resources()?;

    for img in &resources.sampled_images {
        let set = refl.get_decoration(img.id, Decoration::DescriptorSet)?;
        let binding = refl.get_decoration(img.id, Decoration::Binding)?;

        let var_name = refl.get_name(img.id)?;

        let img_type = refl.get_type(img.base_type_id)?;

        let dim_raw;
        if let Type::SampledImage { image, .. } = img_type {
            dim_raw = image.dim;
        } else {
            continue;
        }

        let dim = match dim_raw {
            spirv_cross::spirv::Dim::Dim1D => ImageDimension::One,
            spirv_cross::spirv::Dim::Dim2D => ImageDimension::Two,
            spirv_cross::spirv::Dim::Dim3D => ImageDimension::Three,
            spirv_cross::spirv::Dim::DimCube => ImageDimension::Cube,
            spirv_cross::spirv::Dim::DimSubpassData => ImageDimension::SubpassInput,
            _ => continue,
        };

        set_bindings.push(SetBinding {
            set,
            binding,
            data: SetBindingData::SampledImage { dim },
            var_name,
        });
    }

    Ok(())
}

fn reflect_ubos(
    refl: &mut Refl,
    ubos: &[Resource],
    set_bindings: &mut Vec<SetBinding>,
) -> Result<()> {
    for ubo in ubos {
        let set = refl.get_decoration(ubo.id, Decoration::DescriptorSet)?;
        let binding = refl.get_decoration(ubo.id, Decoration::Binding)?;

        let var_name = refl.get_name(ubo.id)?;
        let block_name = refl.get_name(ubo.base_type_id)?;

        let block_type = refl.get_type(ubo.base_type_id)?;
        let block_size = refl.get_declared_struct_size(ubo.base_type_id)?;

        let member_types;
        if let Type::Struct {
            member_types: mt, ..
        } = block_type
        {
            member_types = mt;
        } else {
            // should never happend since every ubo has base_type of Struct but rust requires it
            continue;
        }

        let mut members = Vec::new();
        for (i, type_id) in member_types.iter().enumerate() {
            let offset =
                refl.get_member_decoration(ubo.base_type_id, i as u32, Decoration::Offset)?;
            let size = refl.get_declared_struct_member_size(ubo.base_type_id, i as u32)?;

            let member_type = refl.get_type(*type_id)?;

            let name = refl.get_member_name(ubo.base_type_id, i as u32)?;

            let kind = match member_type {
                Type::Float {
                    vecsize, columns, ..
                } => {
                    if vecsize == 1 && columns == 1 {
                        BlockMemberType::Float
                    } else if columns == 1 {
                        BlockMemberType::FloatVector(vecsize)
                    } else if vecsize == columns {
                        BlockMemberType::FloatMatrix(vecsize)
                    } else {
                        BlockMemberType::Unsupported
                    }
                }
                _ => BlockMemberType::Unsupported,
            };

            members.push(BlockMember {
                kind,
                offset,
                size,
                name,
            });
        }

        set_bindings.push(SetBinding {
            set,
            binding,
            data: SetBindingData::UniformBuffer {
                layout: BlockLayout {
                    members,
                    block_name,
                    total_size: block_size,
                },
            },
            var_name,
        });
    }

    Ok(())
}

pub fn merge(
    a: ShaderInfo,
    b: &ShaderInfo,
    names_must_match: bool,
) -> std::result::Result<ShaderInfo, String> {
    let mut res = a;

    for binding in &b.set_bindings {
        let mut found = false;
        for a_binding in &res.set_bindings {
            if a_binding.set != binding.set || a_binding.binding != binding.binding {
                continue;
            }

            if names_must_match {
                if a_binding != binding {
                    return Err(String::from("Incompatible shaders"));
                }
            } else if !a_binding.equal_ignore_names(binding) {
                return Err(String::from("Incompatible shaders"));
            }

            found = true;
            break;
        }

        if !found {
            res.set_bindings.push(binding.clone());
        }
    }

    Ok(res)
}
