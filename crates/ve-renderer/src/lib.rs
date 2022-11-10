#![warn(
    missing_docs,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::missing_safety_doc
)]

//! This crate is a basic Vulkan Renderer for the vulkan-engine

pub mod error;
pub mod renderer;
pub mod version;
pub mod window;
