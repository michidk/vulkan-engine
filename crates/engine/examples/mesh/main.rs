use std::{path::Path, rc::Rc};

/// Renders a brdf example
use gfx_maths::*;
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
    vulkan_engine::run_engine(1920, 1080, "Mesh Example", setup);
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
        .set_vec4("albedo", Vec4::new(0.5, 0.5, 0.5, 1.0))
        .unwrap();
    brdf_material0.set_float("metallic", 0.0).unwrap();
    brdf_material0.set_float("roughness", 0.1).unwrap();

    let mesh_data = ve_format::mesh::MeshData::from_file(Path::new("./assets/models/cube.vem"))
        .expect("Model cube.vem not found!");

    let mesh = Mesh::bake(
        mesh_data,
        (*engine.vulkan_manager.allocator).clone(),
        &mut engine.vulkan_manager.uploader,
    )
    .expect("Error baking mesh!");

    let model = Model {
        material: brdf_material0,
        mesh,
    };

    let entity = scene.new_entity_with_transform(
        "Cube".to_string(),
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

    scene
        .new_entity_with_transform(
            "DirLight1".to_string(),
            Transform {
                position: Vec3::zero(),
                rotation: Quaternion::axis_angle(Vec3::new(1.0, 0.0, 0.0), -90.0f32.to_radians()),
                scale: Vec3::one(),
            },
        )
        .new_component::<LightComponent>()
        .light
        .set(
            DirectionalLight {
                direction: Vec4::zero(),
                illuminance: Vec4::new(10.1, 10.1, 10.1, 0.0),
            }
            .into(),
        );

    scene
        .new_entity_with_transform(
            "DirLight2".to_string(),
            Transform {
                position: Vec3::zero(),
                rotation: Quaternion::axis_angle(Vec3::new(1.0, 0.0, 0.0), 90.0f32.to_radians()),
                scale: Vec3::one(),
            },
        )
        .new_component::<LightComponent>()
        .light
        .set(
            DirectionalLight {
                direction: Vec4::zero(),
                illuminance: Vec4::new(1.6, 1.6, 1.6, 0.0),
            }
            .into(),
        );

    scene
        .new_entity_with_transform(
            "PointLight White 1".to_string(),
            Transform {
                position: Vec3::new(0.1, -3.0, -3.0),
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
    scene
        .new_entity_with_transform(
            "PointLight White 2".to_string(),
            Transform {
                position: Vec3::new(0.1, -3.0, -3.0),
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
    scene
        .new_entity_with_transform(
            "PointLight White 3".to_string(),
            Transform {
                position: Vec3::new(0.1, -3.0, -3.0),
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
    scene
        .new_entity_with_transform(
            "PointLight White 4".to_string(),
            Transform {
                position: Vec3::new(0.1, -3.0, -3.0),
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

    scene
        .new_entity_with_transform(
            "PointLight Red".to_string(),
            Transform {
                position: Vec3::new(0.0, 0.0, -3.0),
                rotation: Quaternion::identity(),
                scale: Vec3::one(),
            },
        )
        .new_component::<LightComponent>()
        .light
        .set(
            PointLight {
                position: Vec4::zero(),
                luminous_flux: Vec4::new(100.0, 0.0, 0.0, 0.0),
            }
            .into(),
        );

    scene.load();
}
