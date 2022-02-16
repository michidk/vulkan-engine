use env_logger::Env;

use crate::{version::Version, graphics::context::Context};


pub struct AppConfig {
    pub app_info: AppInfo,
    pub engine_config: EngineConfig,
}

pub struct AppInfo {
    pub app_name: &'static str,
    pub app_version: Version,
}

pub struct EngineConfig {
    pub window_width: u32,
    pub window_height: u32,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self { 
            window_width: 800, 
            window_height: 600,
        }
    }
}

pub fn run(app_config: AppConfig) -> ! {
    #[cfg(debug_assertions)]
    let level = "debug";
    #[cfg(not(debug_assertions))]
    let level = "warn";
    env_logger::init_from_env(Env::default().default_filter_or(level));

    let context = Context::new(&app_config.app_info).expect("Failed to create Graphics Context");

    loop {}
}
