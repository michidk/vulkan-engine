use std::process::exit;

/// Renders a brdf example
use crystal::prelude::*;
use log::error;
use ve_format::mesh::{Face, MeshData, Submesh, Vertex};
use vulkan_engine::{scene::{material::*, model::mesh::Mesh}, vulkan::lighting_pipeline::LightingPipeline};
use vulkan_engine::{
    core::window::{self, Dimensions},
    engine::{self, Engine, EngineInit},
    scene::{
        camera::Camera,
        light::{DirectionalLight, PointLight},
        material::MaterialPipeline,
        model::{
            Model,
        },
        transform::Transform,
    },
};

#[repr(C)]
#[derive(MaterialBindingFragment)]
struct BrdfColorData {
    color: Vec4<f32>,
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
    let engine_info = engine::Info {
        window_info: window::Info {
            initial_dimensions: Dimensions {
                width: 1920,
                height: 1080,
            },
            title: "Vulkan BRDF Example",
        },
        app_name: "Vulkan BRDF Example",
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

    let brdf_lighting = LightingPipeline::new(
     Some("deferred_point_brdf"), 
        Some("deferred_directional_brdf"),
        None,
        engine.vulkan_manager.pipeline_layout_resolve_pass,
        engine.vulkan_manager.renderpass,
        engine.vulkan_manager.device.clone(),
        1
    ).unwrap();
    engine.vulkan_manager.register_lighting_pipeline(brdf_lighting.clone());

    let unlit_lighting = LightingPipeline::new(
        None,
        None,
        Some("deferred_unlit"),
        engine.vulkan_manager.pipeline_layout_resolve_pass,
        engine.vulkan_manager.renderpass,
        engine.vulkan_manager.device.clone(),
        2
    ).unwrap();
    engine.vulkan_manager.register_lighting_pipeline(unlit_lighting.clone());

    let brdf_pipeline = MaterialPipeline::<BrdfMaterialData>::new(
        engine.vulkan_manager.device.clone(),
        (*engine.vulkan_manager.allocator).clone(),
        "material_solid_color",
        engine.vulkan_manager.desc_layout_frame_data,
        engine.vulkan_manager.renderpass,
        brdf_lighting.as_ref()
    )
    .unwrap();
    let brdf_material0 = brdf_pipeline
        .create_material(BrdfMaterialData {
            color_data: BrdfColorData {
                color: Vec4::new(0.5, 0.5, 0.5, 1.0),
                metallic: 0.0,
                roughness: 0.1,
            },
        })
        .unwrap();
    let brdf_material1 = brdf_pipeline
        .create_material(BrdfMaterialData {
            color_data: BrdfColorData {
                color: Vec4::new(1.0, 0.5, 0.5, 1.0),
                metallic: 0.0,
                roughness: 0.2,
            },
        })
        .unwrap();

    let unlit_pipeline = MaterialPipeline::<BrdfMaterialData>::new(
        engine.vulkan_manager.device.clone(),
        (*engine.vulkan_manager.allocator).clone(),
        "material_solid_color",
        engine.vulkan_manager.desc_layout_frame_data,
        engine.vulkan_manager.renderpass,
        unlit_lighting.as_ref()
    ).unwrap();
    let unlit_material0 = unlit_pipeline.create_material(
        BrdfMaterialData {
            color_data: BrdfColorData {
                color: Vec4::new(1.0, 0.0, 1.0, 1.0),
                metallic: 0.0,
                roughness: 0.0
            }
        }
    ).unwrap();

    let mesh_data0 = MeshData {
        vertices: vec![
            Vertex {
                position: Vec3::new(-1.0, -1.0, 0.0),
                color: Vec3::new(1.0, 0.0, 1.0),
                normal: Vec3::new(0.0, 0.0, -1.0),
                uv: Vec2::new(0.0, 0.0),
            },
            Vertex {
                position: Vec3::new(1.0, -1.0, 0.0),
                color: Vec3::new(1.0, 0.0, 1.0),
                normal: Vec3::new(0.0, 0.0, -1.0),
                uv: Vec2::new(0.0, 0.0),
            },
            Vertex {
                position: Vec3::new(0.0, 1.0, 0.0),
                color: Vec3::new(1.0, 0.0, 1.0),
                normal: Vec3::new(0.0, 0.0, -1.0),
                uv: Vec2::new(0.0, 0.0),
            },
        ],
        submeshes: vec![Submesh {
            faces: vec![Face { indices: [0, 1, 2] }],
        }],
    };
    let mesh0 = Mesh::bake(
            mesh_data0,
            (*engine.vulkan_manager.allocator).clone()
        )
        .unwrap();

    let mesh_data1 = MeshData {
        vertices: vec![
            Vertex {
                position: Vec3::new(-0.5, -1.0, 0.0),
                color: Vec3::new(1.0, 0.0, 1.0),
                normal: Vec3::new(0.0, 0.0, -1.0),
                uv: Vec2::new(0.0, 0.0),
            },
            Vertex {
                position: Vec3::new(1.0, -1.0, 0.0),
                color: Vec3::new(1.0, 0.0, 1.0),
                normal: Vec3::new(0.0, 0.0, -1.0),
                uv: Vec2::new(0.0, 0.0),
            },
            Vertex {
                position: Vec3::new(0.0, 1.0, 0.0),
                color: Vec3::new(1.0, 0.0, 1.0),
                normal: Vec3::new(0.0, 0.0, -1.0),
                uv: Vec2::new(0.0, 0.0),
            },
        ],
        submeshes: vec![Submesh {
            faces: vec![Face { indices: [0, 1, 2] }],
        }],
    };
    let mesh1 = Mesh::bake(
            mesh_data1,
            (*engine.vulkan_manager.allocator).clone()
        )
        .unwrap();

    let model = Model {
        material: brdf_material0.clone(),
        mesh: mesh0.clone(),
        transform: Transform {
            position: Vec3::new(0.0, 0.0, 5.0),
            rotation: Quaternion::from_axis_angle(Unit::new_normalize(Vec3::new(0.0, 0.0, 1.0)), Angle::from_deg(0.0)),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    };
    let model2 = Model {
        material: brdf_material1.clone(),
        mesh: mesh0.clone(),
        transform: Transform {
            position: Vec3::new(-3.0, 0.0, 4.0),
            rotation: Quaternion::from_axis_angle(Unit::new_normalize(Vec3::new(0.0, 0.0, 1.0)), Angle::from_deg(15.0)),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    };
    let model3 = Model {
        material: brdf_material0.clone(),
        mesh: mesh1.clone(),
        transform: Transform {
            position: Vec3::new(3.0, 0.0, 5.0),
            rotation: Quaternion::from_axis_angle(Unit::new_normalize(Vec3::new(0.0, 0.0, 1.0)), Angle::from_deg(-15.0)),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    };
    let model4 = Model {
        material: unlit_material0.clone(),
        mesh: mesh0.clone(),
        transform: Transform {
            position: Vec3::new(2.0, 1.0, 4.0),
            rotation: Quaternion::from_axis_angle(Unit::new_normalize(Vec3::new(0.0, 0.0, 1.0)), Angle::from_deg(180.0)),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    };
    let model5 = Model {
        material: brdf_material0.clone(),
        mesh: mesh0.clone(),
        transform: Transform {
            position: Vec3::new(-2.5, 1.0, 4.0),
            rotation: Quaternion::from_axis_angle(Unit::new_normalize(Vec3::new(0.0, 0.0, 1.0)), Angle::from_deg(15.0)),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    };

    scene.add(model);
    scene.add(model2);
    scene.add(model3);
    scene.add(model4);
    scene.add(model5);

    // setup scene
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
