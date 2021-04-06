use ash::vk;

#[derive(thiserror::Error, Debug)]
pub enum VulkanError {
    #[error("Unknown error")]
    Unknown,
    #[error("Vulkan error: {0}")]
    VkError(#[from] vk::Result),
    #[error("VulkanMemory error: {0}")]
    VkMemError(#[from] vk_mem::error::Error),
    #[error("No suitable gpu found")]
    NoSuitableGpu,
    #[error("No suitable queue family found")]
    NoSuitableQueueFamily,
    #[error("Invalid handle")]
    InvalidHandle,
}
