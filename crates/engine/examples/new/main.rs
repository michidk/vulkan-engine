use vulkan_engine::{AppConfig, AppInfo, version::Version, EngineConfig};


fn main() {
    vulkan_engine::run(AppConfig {
        app_info: AppInfo {
            app_name: "New Example",
            app_version: Version(0, 1, 0),
        },
        engine_config: EngineConfig {
            window_width: 1920,
            window_height: 1080,
        },
    });
}
