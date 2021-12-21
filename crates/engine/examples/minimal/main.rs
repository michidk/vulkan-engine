/// A minimal example that just initializes the engine but does not display anything
use std::{process::exit, rc::Rc};

use gfx_maths::*;
use log::error;
use ve_format::mesh::{Face, MeshData, Submesh, Vertex};
use vulkan_engine::{
    core::{
        engine::{self, Engine, EngineInit},
        window::{self, Dimensions},
    },
    scene::{
        component::{camera_component::CameraComponent, renderer::RendererComponent},
        material::MaterialPipeline,
        model::{mesh::Mesh, Model},
        transform::Transform,
    },
    vulkan::lighting_pipeline::LightingPipeline,
};

fn main() {
    // setting up logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    // initialize engine
    let engine_info = engine::EngineInfo {
        window_info: window::InitialWindowInfo {
            initial_dimensions: Dimensions {
                width: 1920,
                height: 1080,
            },
            title: "Vulkan Minimal Example",
        },
        app_name: "Vulkan Minimal Example",
    };

    // setup engine
    let engine_engine = EngineInit::new(engine_info);

    // start engine
    match engine_engine {
        Ok(mut engine_engine) => {
            setup(&mut engine_engine.engine);
            engine_engine.start();
        }
        Err(err) => {
            error!("{}", err);
            exit(1);
        }
    }
}

fn setup(engine: &mut Engine) {
    let scene = &mut engine.scene;

    let lighting_pipeline = LightingPipeline::new(
        None,
        None,
        Some("deferred_unlit"),
        engine.vulkan_manager.pipeline_layout_resolve_pass,
        engine.vulkan_manager.renderpass,
        engine.vulkan_manager.device.clone(),
        1,
    )
    .unwrap();
    engine
        .vulkan_manager
        .register_lighting_pipeline(lighting_pipeline.clone());

    let pipeline = MaterialPipeline::new(
        engine.vulkan_manager.device.clone(),
        (*engine.vulkan_manager.allocator).clone(),
        "vertex_unlit",
        engine.vulkan_manager.desc_layout_frame_data,
        engine.vulkan_manager.renderpass,
        lighting_pipeline.as_ref(),
    )
    .unwrap();
    let material0 = pipeline.create_material().unwrap();

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

    let mesh = Mesh::bake(
        mesh_data,
        (*engine.vulkan_manager.allocator).clone(),
        &mut engine.vulkan_manager.uploader,
    )
    .unwrap();

    let model = Model {
        material: material0,
        mesh,
    };

    let entity = scene.new_entity_with_transform(
        "Quad".to_owned(),
        Transform {
            position: Vec3::new(0.0, 0.0, 5.0),
            rotation: Quaternion::axis_angle(Vec3::new(1.0, 0.0, 0.0), 0.0f32.to_radians()),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    let comp = entity.new_component::<RendererComponent>();
    *comp.model.borrow_mut() = Some(Rc::new(model));

    let main_cam = scene.new_entity_with_transform(
        "Main Camera".to_owned(),
        Transform {
            position: Vec3::new(0.0, 0.0, -5.0),
            rotation: Quaternion::identity(),
            scale: Vec3::one(),
        },
    );
    main_cam.new_component::<CameraComponent>();

    scene.load();

    // scene.add(Model {
    //     material: material0,
    //     mesh,
    //     transform: Transform {
    //         position: Vec3::new(0.0, 0.0, 5.0),
    //         rotation: Quaternion::new(0.0, 0.0, 0.0, 1.0),
    //         scale: Vec3::new(1.0, 1.0, 1.0),
    //     },
    // });
}
