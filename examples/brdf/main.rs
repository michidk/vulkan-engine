use std::process::exit;

/// Renders a brdf example
use crystal::prelude::{Mat4, Quaternion, Vec2, Vec3, Vec4};
use log::error;
use vulkan_engine::scene::material::*;
use vulkan_engine::{
    core::window::{self, Dimensions},
    engine::{self, Engine, EngineInit},
    scene::{
        camera::Camera,
        light::{DirectionalLight, PointLight},
        material::MaterialPipeline,
        model::{
            mesh::{Face, MeshData, Submesh, Vertex},
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

    let brdf_pipeline = MaterialPipeline::<BrdfMaterialData>::new(
        engine.vulkan_manager.device.clone(),
        (*engine.vulkan_manager.allocator).clone(),
        "brdf",
        engine.vulkan_manager.desc_layout_frame_data,
        engine.vulkan_manager.renderpass,
        engine.vulkan_manager.swapchain.extent.width,
        engine.vulkan_manager.swapchain.extent.height,
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

    let mesh_data = MeshData {
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

    let transform = Mat4::translate(Vec3::new(0.0, 0.0, 5.0));
    let inv_transform = Mat4::translate(Vec3::new(0.0, 0.0, -5.0));
    let mesh = mesh_data
        .bake(
            (*engine.vulkan_manager.allocator).clone(),
            transform,
            inv_transform,
        )
        .unwrap();

    let model = Model {
        material: brdf_material0,
        mesh: mesh,
        transform: Transform {
            position: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::new(0.0, 0.0, 0.0, 1.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    };

    scene.add(model);

    // setup scene
    let lights = &mut scene.light_manager;
    lights.add_light(DirectionalLight {
        direction: Vec3::new(0., -1., 0.),
        illuminance: Vec3::new(10.1, 10.1, 10.1),
    });
    lights.add_light(DirectionalLight {
        direction: Vec3::new(0., 1., 0.),
        illuminance: Vec3::new(1.6, 1.6, 1.6),
    });
    lights.add_light(PointLight {
        position: Vec3::new(0.1, -3.0, -3.0),
        luminous_flux: Vec3::new(100.0, 100.0, 100.0),
    });
    lights.add_light(PointLight {
        position: Vec3::new(0.1, -3.0, -3.0),
        luminous_flux: Vec3::new(100.0, 100.0, 100.0),
    });
    lights.add_light(PointLight {
        position: Vec3::new(0.1, -3.0, -3.0),
        luminous_flux: Vec3::new(100.0, 100.0, 100.0),
    });
    lights.add_light(PointLight {
        position: Vec3::new(0.1, -3.0, -3.0),
        luminous_flux: Vec3::new(100.0, 100.0, 100.0),
    });
    lights.add_light(PointLight {
        position: Vec3::new(0.0, 0.0, -3.0),
        luminous_flux: Vec3::new(100.0, 0.0, 0.0),
    });

    // let mat = Material::new(
    //     "brdf",
    //     ShaderData<2> {
    //         [
    //             ShaderDataType::Uniform(),
    //             ShaderDataType::Uniform(),
    //         ]
    //     }
    // );

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
