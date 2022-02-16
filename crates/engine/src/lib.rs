
#[macro_use]
mod profiler_macros;

pub mod old;

pub mod version;

mod startup;
use env_cast::env_cast;
pub use startup::*;

use version::Version;

pub(crate) mod graphics;

pub const ENGINE_NAME: &'static str = "vulkan-engine";

pub const ENGINE_VERSION: Version = Version(
    env_cast!("CARGO_PKG_VERSION_MAJOR" as u32),
    env_cast!("CARGO_PKG_VERSION_MINOR" as u32),
    env_cast!("CARGO_PKG_VERSION_PATCH" as u32),
);
