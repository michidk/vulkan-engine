use std::process::exit;

use env_logger::Env;
use prelude::core::{
    engine::{self, EngineInit},
    window::Dimensions,
};

#[macro_use]
pub mod profiler_macros;
pub mod assets;
pub mod core;
pub mod scene;
pub mod utils;
pub mod vulkan;

pub use engine_derive::Component;

pub mod prelude {
    pub use crate::assets;
    pub use crate::core;
    pub use crate::scene;
    pub use crate::utils;
    pub use crate::vulkan;
    pub use gfx_maths::*;
}

pub fn run_engine<Init: FnOnce(&mut core::engine::Engine)>(
    width: u32,
    height: u32,
    app_name: &'static str,
    init_func: Init,
) -> ! {
    #[cfg(debug_assertions)]
    let level = "debug";
    #[cfg(not(debug_assertions))]
    let level = "warn";
    env_logger::init_from_env(Env::default().default_filter_or(level));

    // initialize engine
    let engine_info = engine::EngineInfo {
        window_info: core::window::InitialWindowInfo {
            initial_dimensions: Dimensions { width, height },
            title: app_name,
        },
        app_name,
    };

    // setup engine
    let engine_init = EngineInit::new(engine_info);

    // start engine
    match engine_init {
        Ok(mut engine_init) => {
            init_func(&mut engine_init.engine);
            engine_init.start();
        }
        Err(err) => {
            log::error!("{}", err);
            exit(1);
        }
    }
}
