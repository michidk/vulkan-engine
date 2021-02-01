use vulkan_renderer::parser;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();
    parser::parse("assets/polygon.obj");
}
