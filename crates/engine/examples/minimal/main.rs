/// A minimal example that just initializes the engine but does not display anything
use std::rc::Rc;

use gfx_maths::*;
use ve_format::mesh::{Face, MeshData, Submesh, Vertex};
use vulkan_engine::{
    core::engine::Engine,
    scene::{
        component::{
            camera_component::CameraComponent, debug_movement_component::DebugMovementComponent,
            renderer::RendererComponent,
        },
        material::MaterialPipeline,
        model::{mesh::Mesh, Model},
        transform::Transform,
    },
    vulkan::lighting_pipeline::LightingPipeline,
};

fn main() {
    vulkan_engine::run_engine(1920, 1080, "Minimal Example", setup);
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
    main_cam.new_component::<DebugMovementComponent>();

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
