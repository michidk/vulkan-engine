use std::env;
use std::fs;
use std::process::Command;

fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed=../../shaders/");

    let engine_dir = env::current_dir().unwrap(); // .
    let project_dir = engine_dir.parent().unwrap().parent().unwrap(); // ../../
    let shader_source_dir = project_dir.join("shaders"); // ../../shaders
    let asset_dir = project_dir.join("assets"); // ../../assets
    let shader_target_dir = asset_dir.join("shaders"); // ../../assets/shaders

    // create required directories
    fs::create_dir_all(&shader_target_dir).unwrap();

    // compile shaders
    let paths = fs::read_dir(shader_source_dir).unwrap();
    for path in paths {
        let path = path.as_ref().unwrap().path();
        if let Some(ext) = path.extension() {
            if ext == "hlsl" {
                println!(
                    "Compiling shader {}",
                    &path.file_name().unwrap().to_str().unwrap()
                );

                let _ = compile_shader(&shader_target_dir, &path, "vert");
                let _ = compile_shader(&shader_target_dir, &path, "frag");
            }
        }
    }
}

fn compile_shader(
    target_dir: &std::path::Path,
    source_path: &std::path::Path,
    shader_type: &str,
) -> std::process::Output {
    let output_path = target_dir.join(format!(
        "{}-{}.spv",
        &source_path.file_stem().unwrap().to_str().unwrap(),
        shader_type
    ));

    let output = Command::new("glslc")
        .arg("--target-env=vulkan1.2")
        .arg("-fauto-combined-image-sampler")
        .arg(format!("-fshader-stage={shader_type}"))
        .arg(format!("-fentry-point={shader_type}"))
        .arg(&source_path.display().to_string())
        .arg(format!("-o{}", output_path.to_string_lossy()))
        .output()
        .expect("failed to compile shaders using glslc");

    println!("Shader Compiler Output: {output:#?}");
    assert!(output.status.success());

    output
}
