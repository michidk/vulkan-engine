use std::process::exit;

use env_logger::Env;
use prelude::core::{
    engine::{self, EngineInit},
    window::Dimensions,
};

pub mod assets;
pub mod core;
pub mod scene;
pub mod utils;
pub mod vulkan;

pub mod prelude {
    pub use crate::old::assets;
    pub use crate::old::core;
    pub use crate::old::scene;
    pub use crate::old::utils;
    pub use crate::old::vulkan;
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
