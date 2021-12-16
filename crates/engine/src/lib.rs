#![feature(map_first_last)]
#![feature(once_cell)]
#![feature(never_type)]
#![feature(arc_new_cyclic)]

pub mod assets;
pub mod core;
pub mod scene;
pub mod utils;
pub mod vulkan;

pub mod prelude {
    pub use crate::assets;
    pub use crate::core;
    pub use crate::scene;
    pub use crate::utils;
    pub use crate::vulkan;
    pub use gfx_maths::*;
}
