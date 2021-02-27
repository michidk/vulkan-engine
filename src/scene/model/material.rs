pub struct UniformData {
    pub size: usize,
}

pub struct TextureData {}

pub enum UniformLayoutBinding {
    Uniform(UniformData),
    Texture(TextureData),
}

impl From<UniformData> for UniformLayoutBinding {
    fn from(value: UniformData) -> Self {
        UniformLayoutBinding::Uniform(value)
    }
}

impl From<TextureData> for UniformLayoutBinding {
    fn from(value: TextureData) -> Self {
        UniformLayoutBinding::Texture(value)
    }
}

pub struct ShaderLayout{
    bindings: Vec<UniformLayoutBinding>,
}
