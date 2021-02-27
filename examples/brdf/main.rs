/// Renders a brdf example
use std::process::exit;

use crystal::prelude::{Mat4, Vec3};
use log::error;
use vulkan_engine::{core::window::{self, Dimensions}, engine::{self, Engine, EngineInit}, scene::{camera::Camera, light::{DirectionalLight, PointLight}, model::Material}, utils::color::Color};

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
        0
    );
}
