use std::rc::Rc;

/// Renders a terrain example
use gfx_maths::*;
use noise::{NoiseFn, OpenSimplex};
use ve_format::mesh::{Face, MeshData, Submesh, Vertex};
use vulkan_engine::{
    core::engine::Engine,
    scene::{
        component::{
            camera_component::CameraComponent, debug_movement_component::DebugMovementComponent,
            light_component::LightComponent, renderer::RendererComponent,
        },
        light::{DirectionalLight, PointLight},
        material::MaterialPipeline,
        model::Model,
        transform::Transform,
    },
};
use vulkan_engine::{
    scene::model::mesh::Mesh,
    vulkan::{lighting_pipeline::LightingPipeline, pp_effect::PPEffect},
};

fn main() {
    vulkan_engine::run_engine(1920, 1080, "Terrain Example", setup);
}

fn build_terrain() -> MeshData {
    let size = 128;
    let height_scale = 10.0;
    let scale = 0.125;

    let mut vertices = Vec::new();
    let mut faces = Vec::new();
    let noise = OpenSimplex::new();

    for x in 0..size {
        for z in 0..size {
            let height = (noise.get([x as f64 * scale, z as f64 * scale]) * height_scale) as f32;

            vertices.push(Vertex {
                position: Vec3::new(x as f32, height, z as f32),
                color: Vec3::one(),
                normal: Vec3::new(0.0, 1.0, 0.0),
                uv: Vec2::zero(),
            });

            if x < size - 1 && z < size - 1 {
                faces.push(Face {
                    indices: [
                        (x + z * size) as u32,
                        (x + (z + 1) * size) as u32,
                        ((x + 1) + (z + 1) * size) as u32,
                    ],
                });
                faces.push(Face {
                    indices: [
                        (x + z * size) as u32,
                        ((x + 1) + (z + 1) * size) as u32,
                        ((x + 1) + z * size) as u32,
                    ],
                });
            }
        }
    }

    MeshData {
        vertices,
        submeshes: vec![Submesh { faces }],
    }
}

fn setup(engine: &mut Engine) {
    let scene = &mut engine.scene;

    let pp_tonemap = PPEffect::new(
        "tone_map",
        engine.vulkan_manager.pipe_layout_pp,
        engine.vulkan_manager.renderpass_pp,
        engine.vulkan_manager.device.clone(),
    )
    .unwrap();
    engine.vulkan_manager.register_pp_effect(pp_tonemap);

    let brdf_resolve_pipeline = LightingPipeline::new(
        Some("deferred_point_brdf"),
        Some("deferred_directional_brdf"),
        None,
        engine.vulkan_manager.pipeline_layout_resolve_pass,
        engine.vulkan_manager.renderpass,
        engine.vulkan_manager.device.clone(),
        1,
    )
    .unwrap();
    engine
        .vulkan_manager
        .register_lighting_pipeline(brdf_resolve_pipeline.clone());

    let brdf_pipeline = MaterialPipeline::new(
        engine.vulkan_manager.device.clone(),
        (*engine.vulkan_manager.allocator).clone(),
        "material_solid_color",
        engine.vulkan_manager.desc_layout_frame_data,
        engine.vulkan_manager.renderpass,
        brdf_resolve_pipeline.as_ref(),
    )
    .unwrap();
    let brdf_material0 = brdf_pipeline.create_material().unwrap();

    brdf_material0
        .set_vec4("albedo", Vec4::new(0.0, 0.5, 0.0, 1.0))
        .unwrap();
    brdf_material0.set_float("metallic", 0.0).unwrap();
    brdf_material0.set_float("roughness", 0.95).unwrap();

    let mesh_data = build_terrain();

    let mesh = Mesh::bake(
        mesh_data,
        (*engine.vulkan_manager.allocator).clone(),
        &mut engine.vulkan_manager.uploader,
        true,
    )
    .expect("Error baking mesh!");

    let model = Model {
        material: brdf_material0,
        mesh,
    };

    let entity = scene.new_entity_with_transform(
        "Terrain".to_string(),
        Transform {
            position: Vec3::zero(),
            rotation: Quaternion::identity(),
            scale: Vec3::one(),
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

    scene
        .new_entity_with_transform(
            "Sun".to_string(),
            Transform {
                position: Vec3::zero(),
                rotation: Quaternion::from_euler_angles_zyx(&Vec3::new(-153.0, 0.0, 0.0)),
                scale: Vec3::one(),
            },
        )
        .new_component::<LightComponent>()
        .light
        .set(
            DirectionalLight {
                direction: Vec4::zero(),
                illuminance: Vec4::new(0.9, 0.9, 0.9, 0.0),
            }
            .into(),
        );

    scene
        .new_entity_with_transform(
            "Point Light".to_string(),
            Transform {
                position: Vec3::new(50.0, 10.0, 50.0),
                rotation: Quaternion::identity(),
                scale: Vec3::one(),
            },
        )
        .new_component::<LightComponent>()
        .light
        .set(
            PointLight {
                position: Vec4::zero(),
                luminous_flux: Vec4::new(100.0, 100.0, 100.0, 0.0),
            }
            .into(),
        );

    scene.load();
}
