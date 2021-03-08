/// A minimal example that just initializes the engine but does not display anything
use std::process::exit;

use crystal::prelude::*;
use log::error;
use vulkan_engine::{
    core::window::{self, Dimensions},
    engine::{self, Engine, EngineInit},
    scene::{
        camera::Camera,
        light::DirectionalLight,
        material::{MaterialBinding, MaterialBindingFragment, MaterialData, MaterialPipeline},
        model::{
            mesh::{Face, MeshData, Submesh, Vertex},
            Model,
        },
        transform::Transform,
    },
};

#[derive(MaterialData)]
struct VertexMaterialData {}

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
        Ok(mut engine_init) => {
            setup(&mut engine_init.engine);
            engine_init.start();
        }
        Err(err) => {
            error!("{}", err);
            exit(1);
        }
    }
}

fn setup(engine: &mut Engine) {
    let scene = &mut engine.scene;

    let pipeline = MaterialPipeline::<VertexMaterialData>::new(
        engine.vulkan_manager.device.clone(),
        (*engine.vulkan_manager.allocator).clone(),
        "vertex_unlit",
        engine.vulkan_manager.desc_layout_frame_data,
        engine.vulkan_manager.renderpass,
        engine.vulkan_manager.swapchain.extent.width,
        engine.vulkan_manager.swapchain.extent.height,
    )
    .unwrap();
    let material0 = pipeline.create_material(VertexMaterialData {}).unwrap();

    // create triangle
    let mesh_data = MeshData {
        vertices: vec![
            Vertex {
                position: Vec3::new(-1.0, -1.0, 0.0),
                color: Vec3::new(1.0, 0.0, 0.0),
                normal: Vec3::new(0.0, 0.0, -1.0),
                uv: Vec2::new(0.0, 0.0),
            },
            Vertex {
                position: Vec3::new(1.0, -1.0, 0.0),
                color: Vec3::new(0.0, 1.0, 0.0),
                normal: Vec3::new(0.0, 0.0, -1.0),
                uv: Vec2::new(0.0, 0.0),
            },
            Vertex {
                position: Vec3::new(0.0, 1.0, 0.0),
                color: Vec3::new(0.0, 0.0, 1.0),
                normal: Vec3::new(0.0, 0.0, -1.0),
                uv: Vec2::new(0.0, 0.0),
            },
        ],
        submeshes: vec![Submesh {
            faces: vec![Face { indices: [0, 1, 2] }],
        }],
    };

    let mesh = mesh_data
        .bake(
            (*engine.vulkan_manager.allocator).clone()
        )
        .unwrap();

    scene.add(Model {
        material: material0,
        mesh: mesh,
        transform: Transform {
            position: Vec3::new(0.0, 0.0, 5.0),
            rotation: Quaternion::new(0.0, 0.0, 0.0, 1.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    });

    // setup camera
    Camera::builder()
        //.fovy(30.0.deg())
        .position(Vec3::new(0.0, 0.0, -5.0))
        .aspect(
            engine.info.window_info.initial_dimensions.width as f32
                / engine.info.window_info.initial_dimensions.height as f32,
        )
        .build()
        .update_buffer(
            &engine.vulkan_manager.allocator,
            &mut engine.vulkan_manager.uniform_buffer,
            0,
        );
}
