/// A minimal example that just initializes the engine but does not display anything
use std::{process::exit, rc::Rc};

use gfx_maths::*;
use log::error;
use ve_format::mesh::{Face, MeshData, Submesh, Vertex};
use vulkan_engine::{
    core::{
        camera::Camera,
        engine::{self, Engine, EngineInit},
        window::{self, Dimensions},
    },
    scene::{
        component::renderer::RendererComponent,
        entity::Entity,
        light::*,
        material::MaterialPipeline,
        model::{mesh::Mesh, Model},
        transform::Transform,
    },
    vulkan::lighting_pipeline::LightingPipeline,
    vulkan::pp_effect::PPEffect,
    vulkan::texture::{Texture2D, TextureFilterMode},
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

    let camera = Camera::builder()
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

    let pp_tonemap = PPEffect::new(
        "tone_map",
        engine.vulkan_manager.pipe_layout_pp,
        engine.vulkan_manager.renderpass_pp,
        engine.vulkan_manager.device.clone(),
    )
    .unwrap();
    engine.vulkan_manager.register_pp_effect(pp_tonemap);

    let lighting_pipeline = LightingPipeline::new(
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
        .register_lighting_pipeline(lighting_pipeline.clone());

    let pipeline = MaterialPipeline::new(
        engine.vulkan_manager.device.clone(),
        (*engine.vulkan_manager.allocator).clone(),
        "material_albedo_tex",
        engine.vulkan_manager.desc_layout_frame_data,
        engine.vulkan_manager.renderpass,
        lighting_pipeline.as_ref(),
    )
    .unwrap();

    let pixels = [
        255u8, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 0, 255, 255,
    ];
    let albedo_tex = Texture2D::new(
        2,
        2,
        &pixels,
        TextureFilterMode::Nearest,
        (*engine.vulkan_manager.allocator).clone(),
        &mut engine.vulkan_manager.uploader,
        engine.vulkan_manager.device.clone(),
    )
    .unwrap();

    let material0 = pipeline.create_material().unwrap();
    material0.set_float("metallic", 0.0).unwrap();
    material0.set_float("roughness", 0.5).unwrap();
    material0.set_texture("u_AlbedoTex", albedo_tex).unwrap();

    // create triangle
    let mesh_data = MeshData {
        vertices: vec![
            Vertex {
                position: Vec3::new(-1.0, -1.0, 0.0),
                color: Vec3::new(1.0, 0.0, 0.0),
                normal: Vec3::new(0.0, 0.0, -1.0),
                uv: Vec2::new(0.0, 1.0),
            },
            Vertex {
                position: Vec3::new(1.0, -1.0, 0.0),
                color: Vec3::new(0.0, 1.0, 0.0),
                normal: Vec3::new(0.0, 0.0, -1.0),
                uv: Vec2::new(1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(0.0, 1.0, 0.0),
                color: Vec3::new(0.0, 0.0, 1.0),
                normal: Vec3::new(0.0, 0.0, -1.0),
                uv: Vec2::new(0.5, 0.0),
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

    let entity = Entity::new_with_transform(
        Rc::downgrade(&scene.root_entity),
        "Quad".to_owned(),
        Transform {
            position: Vec3::new(0.0, 0.0, 10.0),
            rotation: Quaternion::new(0.0, 0.0, 0.0, 1.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    let comp = RendererComponent::new(Rc::new(model));
    entity.add_component(comp);
    scene.add_entity(entity);

    scene.load();

    let lights = &scene.light_manager;
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
