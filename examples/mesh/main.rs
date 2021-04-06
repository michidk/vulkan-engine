use std::{path::Path, process::exit};

/// Renders a brdf example
use crystal::prelude::*;
use log::error;
use vulkan_engine::{
    core::window::{self, Dimensions},
    engine::{self, Engine, EngineInit},
    scene::{
        camera::Camera,
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
    // setting up logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    // initialize engine
    let engine_info = engine::Info {
        window_info: window::Info {
            initial_dimensions: Dimensions {
                width: 1920,
                height: 1080,
            },
            title: "Vulkan Mesh Example",
        },
        app_name: "Vulkan Mesh Example",
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

    let mesh_data = ve_format::mesh::MeshData::from_file(Path::new("./assets/models/sphere.vem"))
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
        transform: Transform {
            position: Vec3::new(0.0, 0.0, 5.0),
            rotation: Quaternion::from_axis_angle(
                Unit::new_normalize(Vec3::new(1.0, 0.0, 0.0)),
                Angle::from_deg(180.0),
            ),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    };

    scene.add(model);

    // setup scene
    let lights = &mut scene.light_manager;
    lights.add_light(DirectionalLight {
        direction: Vec4::new(-1., 1., -1., 0.0),
        illuminance: Vec4::new(10.1, 10.1, 10.1, 0.0),
    });
    lights.add_light(DirectionalLight {
        direction: Vec4::new(0., 1., 0., 0.0),
        illuminance: Vec4::new(1.6, 1.6, 1.6, 0.0),
    });
    lights.add_light(PointLight {
        position: Vec4::new(0.1, -3.0, -3.0, 0.0),
        luminous_flux: Vec4::new(100.0, 100.0, 100.0, 0.0),
    });
    lights.add_light(PointLight {
        position: Vec4::new(0.1, -3.0, -3.0, 0.0),
        luminous_flux: Vec4::new(100.0, 100.0, 100.0, 0.0),
    });
    lights.add_light(PointLight {
        position: Vec4::new(0.1, -3.0, -3.0, 0.0),
        luminous_flux: Vec4::new(100.0, 100.0, 100.0, 0.0),
    });
    lights.add_light(PointLight {
        position: Vec4::new(0.1, -3.0, -3.0, 0.0),
        luminous_flux: Vec4::new(100.0, 100.0, 100.0, 0.0),
    });
    lights.add_light(PointLight {
        position: Vec4::new(0.0, 0.0, -3.0, 0.0),
        luminous_flux: Vec4::new(100.0, 0.0, 0.0, 0.0),
    });

    // setup camera
    let camera = Camera::builder()
        //.fovy(30.0.deg())
        .position(Vec3::new(0.0, 0.0, -5.0))
        .aspect(
            engine.info.window_info.initial_dimensions.width as f32
                / engine.info.window_info.initial_dimensions.height as f32,
        )
        .build();

    camera.update_buffer(
        &engine.vulkan_manager.allocator,
        &mut engine.vulkan_manager.uniform_buffer,
        0,
    );
}
