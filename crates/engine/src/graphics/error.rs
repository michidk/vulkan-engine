use thiserror::Error;


#[derive(Debug, Error)]
pub(crate) enum GraphicsError {
    #[error("Vulkan Unavailable")]
    VulkanUnavailable,
    #[error("Vulkan Surface not supported")]
    SurfaceNotSupported,
    #[error("No suitable physical device")]
    NoDevice,
    #[error("Vulkan API Error: {0}")]
    Vk(#[from] ash::vk::Result),
    #[error("Window creation failed")]
    WindowCreationFailed,
    #[error("Window is minimized")]
    WindowMinimized,
}

pub(crate) type GraphicsResult<T> = ::std::result::Result<T, GraphicsError>;
