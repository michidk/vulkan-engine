//! This module contains all errors that can be returned by functions in this crate.

use ash::vk;
use thiserror::Error;

use crate::version::Version;

/// Contains all possible errors returned by [`Renderer::new()`](crate::renderer::Renderer::new())
#[derive(Debug, Error)]
pub enum CreationError {
    /// [`ash::Entry::load()`] returned an error.
    #[error("ash initialization failed: {0}")]
    LoadingError(#[from] ash::LoadingError),
    /// The Vulkan driver does not support Vulkan 1.2 instance functionality
    #[error("unsupported Vulkan instance version: {0}")]
    UnsupportedInstanceVersion(Version),
    /// A required instance extension is not supported
    #[error("missing instance extension: {0}")]
    MissingInstanceExtension(String),
    /// The Renderer was unable to find a device with the minimum requirements
    #[error("no suitable GPU found")]
    NoDevice,
    /// A Vulkan API function returned an unexpected error
    #[error("Vulkan API error: {0}")]
    Vk(#[from] vk::Result),
}

/// Contains all possible errors returned by [`Window::new()`](create::window::Window::new())
#[derive(Debug, Error)]
pub enum WindowCreationError {
    /// A Vulkan API function returned an unexpected error
    #[error("Vulkan API error: {0}")]
    Vk(#[from] vk::Result),
}
