use vulkan_engine::parser;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();
    let mesh = parser::parse("assets/test.obj")
        .unwrap()
        .build_mesh()
        .unwrap();
    println!("{}", mesh.name.unwrap_or(String::from("no name")));
}
