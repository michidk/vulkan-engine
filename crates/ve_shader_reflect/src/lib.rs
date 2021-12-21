use spirv_layout::Module;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("Reflection error: {0}")]
    ReflectError(#[from] spirv_layout::Error),
    #[error("Incompatible shader property names: {0} and {1}")]
    IncompatiblePropertyNames(String, String),
    #[error("Incompatible shader properties: {0} and {1}")]
    IncompatibleProperties(String, String),
}

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

type Result<T> = std::result::Result<T, Error>;

pub fn reflect_shader(spv_code: &[u32]) -> Result<ShaderInfo> {
    let module = Module::from_words(spv_code)?;

    let set_bindings = reflect_shader_bindings(&module)?;

    Ok(ShaderInfo {
        set_bindings,
        push_constants: None,
    })
}

fn reflect_shader_bindings(module: &Module) -> Result<Vec<SetBinding>> {
    let mut set_bindings = Vec::new();

    for var in module.get_uniforms() {
        let set = var.set.unwrap();
        let binding = var.binding.unwrap();
        let var_name = var.name.clone().unwrap_or_else(|| "".to_owned());

        let data = match module.get_type(var.type_id).unwrap() {
            spirv_layout::Type::Image2D { .. } => SetBindingData::Image {
                dim: ImageDimension::Two,
            },
            spirv_layout::Type::Sampler => SetBindingData::Sampler,
            spirv_layout::Type::SampledImage { .. } => SetBindingData::SampledImage {
                dim: ImageDimension::Two,
            },
            spirv_layout::Type::Struct { elements } => {
                let total_size = module.get_type_size(var.type_id).unwrap();
                let mut members = Vec::new();

                for member in elements {
                    let kind = match module.get_type(member.type_id).unwrap() {
                        spirv_layout::Type::Float32 => BlockMemberType::Float,
                        spirv_layout::Type::Vec2 => BlockMemberType::FloatVector(2),
                        spirv_layout::Type::Vec3 => BlockMemberType::FloatVector(3),
                        spirv_layout::Type::Vec4 => BlockMemberType::FloatVector(4),
                        spirv_layout::Type::Mat3 => BlockMemberType::FloatMatrix(3),
                        spirv_layout::Type::Mat4 => BlockMemberType::FloatMatrix(4),
                        _ => BlockMemberType::Unsupported,
                    };

                    members.push(BlockMember {
                        kind,
                        offset: member.offset.unwrap(),
                        size: 0,
                        name: member.name.clone().unwrap(),
                    });
                }

                SetBindingData::UniformBuffer {
                    layout: BlockLayout {
                        members,
                        block_name: "".to_owned(),
                        total_size,
                    },
                }
            }
            _ => SetBindingData::Sampler,
        };

        set_bindings.push(SetBinding {
            set,
            binding,
            data,
            var_name,
        });
    }

    Ok(set_bindings)
}

pub fn merge(a: ShaderInfo, b: &ShaderInfo, names_must_match: bool) -> Result<ShaderInfo> {
    let mut res = a;

    for binding in &b.set_bindings {
        let mut found = false;
        for a_binding in &res.set_bindings {
            if a_binding.set != binding.set || a_binding.binding != binding.binding {
                continue;
            }

            if names_must_match {
                if a_binding != binding {
                    return Err(Error::IncompatiblePropertyNames(
                        binding.var_name.clone(),
                        a_binding.var_name.clone(),
                    ));
                }
            } else if !a_binding.equal_ignore_names(binding) {
                return Err(Error::IncompatibleProperties(
                    binding.var_name.clone(),
                    a_binding.var_name.clone(),
                ));
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
