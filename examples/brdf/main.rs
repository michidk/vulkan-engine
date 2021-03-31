use std::{path::Path, process::exit};

/// Renders a brdf example
use crystal::prelude::*;
use log::error;
use ve_format::mesh::{Face, MeshData, Submesh, Vertex};
use vulkan_engine::{
    core::{
        camera::Camera,
        window::{self, Dimensions},
    },
    engine::{self, Engine, EngineInit},
    scene::{
        light::{DirectionalLight, PointLight},
        material::MaterialPipeline,
        model::Model,
        transform::Transform,
    },
};
use vulkan_engine::{
    scene::{material::*, model::mesh::Mesh},
    vulkan::{lighting_pipeline::LightingPipeline, pp_effect::PPEffect},
};

#[repr(C)]
#[derive(MaterialBindingFragment)]
struct BrdfColorData {
    albedo: Vec4<f32>,
    metallic: f32,
    roughness: f32,
}

#[derive(MaterialData)]
struct BrdfMaterialData {
    color_data: BrdfColorData,
}

fn main() {
    // setting up logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    // initialize engine
    let engine_info = engine::EngineInfo {
        window_info: window::WindowInfo {
            initial_dimensions: Dimensions {
                width: 1920,
                height: 1080,
            },
            title: "Vulkan BRDF Example",
        },
        app_name: "Vulkan BRDF Example",
    };

    // setup camera
    let camera = Camera::builder()
        //.fovy(30.0.deg())
        .position(Vec3::new(0.0, 0.0, -5.0))
        .aspect(
            engine_info.window_info.initial_dimensions.width as f32
                / engine_info.window_info.initial_dimensions.height as f32,
        )
        .build();

    // setup engine
    let engine_init = EngineInit::new(engine_info, camera);

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

    // pipeline setup
    let pp_tonemap = PPEffect::new(
        "tone_map",
        engine.vulkan_manager.pipe_layout_pp,
        engine.vulkan_manager.renderpass_pp,
        engine.vulkan_manager.device.clone(),
    )
    .unwrap();
    engine.vulkan_manager.register_pp_effect(pp_tonemap.clone());

    let brdf_lighting = LightingPipeline::new(
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
        .register_lighting_pipeline(brdf_lighting.clone());

    let brdf_pipeline = MaterialPipeline::<BrdfMaterialData>::new(
        engine.vulkan_manager.device.clone(),
        (*engine.vulkan_manager.allocator).clone(),
        "material_solid_color",
        engine.vulkan_manager.desc_layout_frame_data,
        engine.vulkan_manager.renderpass,
        brdf_lighting.as_ref(),
    )
    .unwrap();

    let mesh_data_sphere_smooth =
        ve_format::mesh::MeshData::from_file(Path::new("./assets/models/sphere_smooth.vem"))
            .expect("Model sphere_smooth.vem not found!");
    let mesh_sphere_smooth = Mesh::bake(
        mesh_data_sphere_smooth,
        (*engine.vulkan_manager.allocator).clone(),
    )
    .unwrap();

    // setup models
    for x in 0..11 {
        for y in 0..11 {
            let material = brdf_pipeline
                .create_material(BrdfMaterialData {
                    color_data: BrdfColorData {
                        albedo: Vec4::new(0.5, 0.5, 0.5, 0.0),
                        metallic: (x as f32) * 0.1,
                        roughness: (y as f32) * 0.1,
                    },
                })
                .unwrap();

            let model = Model {
                material,
                mesh: mesh_sphere_smooth.clone(),
                transform: Transform {
                    position: Vec3::new(x as f32 - 5.0, y as f32 - 5.0, 10.0),
                    rotation: Quaternion::new(0.0, 0.0, 0.0, 1.0),
                    scale: Vec3::new(0.5, 0.5, 0.5),
                },
            };

            scene.add(model);
        }
    }

    // setup lights
    let lights = &mut scene.light_manager;
    lights.add_light(DirectionalLight {
        direction: Vec4::new(0., 1., 0., 0.0),
        illuminance: Vec4::new(10.1, 10.1, 10.1, 0.0),
    });
    lights.add_light(DirectionalLight {
        direction: Vec4::new(0., -1., 0., 0.0),
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
}
