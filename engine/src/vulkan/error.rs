use ash::vk;

/// Error type for every function in the graphics system
#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum GraphicsError {
    /// General Vulkan API errors
    #[error("Vulkan API error: {0}")]
    Vk(#[from] vk::Result),
    /// Vulkan memory allocator errors
    ///
    /// [`vk_mem::ErrorKind::Vulkan`] will be converted to [`GraphicsError::Vk`].
    #[error("vk_mem error: {0}")]
    VkMem(vk_mem::Error),
    /// Error during shader property discovery
    #[error("Shader reflection error: {0}")]
    ShaderReflect(#[from] ve_shader_reflect::Error),
    /// An invalid property name was given to a Material::set_X() function.
    #[error("Invalid material property: {0}")]
    InvalidMaterialProperty(String),
    /// A Material::set_X() function was called with a property of a different type.
    #[error("Incompatible material property type: {0}")]
    InvalidMaterialPropertyType(String),
    /// No GPU was found that matches the engines requirements
    #[error("No suitable gpu found")]
    NoSuitableGpu,
    /// No queue family was found that matches the engines requirements
    #[error("No suitable queue family found")]
    NoSuitableQueueFamily,
    /// All errors for which no specific variant is available
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<vk_mem::Error> for GraphicsError {
    fn from(err: vk_mem::Error) -> Self {
        match err.kind() {
            vk_mem::ErrorKind::Vulkan(code) => GraphicsError::Vk(*code),
            _ => GraphicsError::VkMem(err),
        }
    }
}

pub type GraphicsResult<T> = Result<T, GraphicsError>;