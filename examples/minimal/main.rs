/// A minimal example that just initializes the engine but does not display anything
use std::process::exit;

use log::error;
use vulkan_engine::{
    core::window::{self, Dimensions},
    engine::{self, EngineInit},
};

fn main() {
    // setting up logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    // initialize engine
    let engine_info = engine::Info {
        window_info: window::Info {
            initial_dimensions: Dimensions {
                width: 1920,
                height: 1080,
            },
            title: "Vulkan Minimal Example",
        },
        app_name: "Vulkan Minimal Example",
    };

    // setup engine
    let engine_init = EngineInit::new(engine_info);

    // start engine
    match engine_init {
        Ok(engine_init) => {
            engine_init.start();
        }
        Err(err) => {
            error!("{}", err);
            exit(1);
        }
    }
}
