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
    ShaderReflect(String),
    /// An invalid property name was given to a Material::set_X() function.
    #[error("Invalid material property: {0}")]
    InvalidMaterialProperty(String),
    /// A Material::set_X() function was called with a property of a different type.
    #[error("Incompatible material property type: {0}")]
    InvalidMaterialPropertyType(String),
    #[error("No suitable gpu found")]
    NoSuitableGpu,
    #[error("No suitable queue family found")]
    NoSuitableQueueFamily,
    #[error("Invalid handle")]
    InvalidHandle,
    /// All errors for which no specific variant is available
    #[error("Miscellaneous error: {0}")]
    Misc(String),
}

impl From<vk_mem::Error> for GraphicsError {
    fn from(err: vk_mem::Error) -> Self {
        match err.kind() {
            vk_mem::ErrorKind::Vulkan(code) => GraphicsError::Vk(*code),
            _ => GraphicsError::VkMem(err),
        }
    }
}

impl From<ve_shader_reflect::Error> for GraphicsError {
    fn from(err: ve_shader_reflect::Error) -> Self {
        GraphicsError::ShaderReflect(err.to_string())
    }
}

impl From<ash::LoadingError> for GraphicsError {
    fn from(err: ash::LoadingError) -> Self {
        GraphicsError::Misc(err.to_string())
    }
}
impl From<ash::InstanceError> for GraphicsError {
    fn from(err: ash::InstanceError) -> Self {
        GraphicsError::Misc(err.to_string())
    }
}

pub type GraphicsResult<T> = Result<T, GraphicsError>;
