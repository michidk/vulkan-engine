use std::{
    cell::{Cell, RefCell},
    path::Path,
    process::exit,
    rc::{Rc, Weak},
};

/// Renders a brdf example
use gfx_maths::*;
use log::error;
use vulkan_engine::{
    core::{
        camera::Camera,
        engine::{self, Engine, EngineInit},
        window::{self, Dimensions},
    },
    scene::{
        component::{renderer::RendererComponent, Component},
        entity::Entity,
        light::{DirectionalLight, PointLight},
        material::MaterialPipeline,
        model::Model,
        transform::Transform,
        Scene,
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
    let engine_info = engine::EngineInfo {
        window_info: window::InitialWindowInfo {
            initial_dimensions: Dimensions {
                width: 1920,
                height: 1080,
            },
            title: "Components Example",
        },
        app_name: "Components Example",
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
    engine.vulkan_manager.register_pp_effect(pp_tonemap);

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

    let brdf_pipeline = MaterialPipeline::new(
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
        &mut engine.vulkan_manager.uploader,
    )
    .unwrap();

    let material_red = brdf_pipeline.create_material().unwrap();
    material_red
        .set_vec4("albedo", Vec4::new(1.0, 0.1, 0.1, 0.0))
        .unwrap();
    material_red.set_float("metallic", 0.1).unwrap();
    material_red.set_float("roughness", 0.7).unwrap();
    let model_red = Rc::new(Model {
        material: material_red,
        mesh: mesh_sphere_smooth.clone(),
    });

    let material_silver = brdf_pipeline.create_material().unwrap();
    material_silver
        .set_vec4("albedo", Vec4::new(0.5, 0.5, 0.5, 0.0))
        .unwrap();
    material_silver.set_float("metallic", 0.5).unwrap();
    material_silver.set_float("roughness", 0.3).unwrap();
    let model_silver = Rc::new(Model {
        material: material_silver,
        mesh: mesh_sphere_smooth,
    });

    {
        let entity_tl = Entity::new_with_transform(
            Rc::downgrade(&scene.root_entity),
            "Top Left Rotating Sphere".to_owned(),
            Transform {
                position: Vec3::new(0.0, 0.0, 10.0),
                rotation: Quaternion::identity(),
                scale: Vec3::one(),
            },
        );
        entity_tl.add_component(RendererComponent::new(model_red));
        entity_tl.add_component(RotateComponent::new());

        let entity_tr = Entity::new_with_transform(
            Rc::downgrade(&entity_tl),
            "Top Right Rotating Sphere".to_owned(),
            Transform {
                position: Vec3::new(3.0, 0.0, 0.0),
                rotation: Quaternion::identity(),
                scale: Vec3::one(),
            },
        );
        entity_tr.add_component(RendererComponent::new(model_silver.clone()));
        entity_tr.add_component(ScaleComponent::new());
        entity_tl.add_child(entity_tr);

        let entity_bl = Entity::new_with_transform(
            Rc::downgrade(&entity_tl),
            "Bottom Left Rotating Sphere".to_owned(),
            Transform {
                position: Vec3::new(0.0, -3.0, 0.0),
                rotation: Quaternion::identity(),
                scale: Vec3::one(),
            },
        );
        entity_bl.add_component(RendererComponent::new(model_silver.clone()));
        entity_bl.add_component(ScaleComponent::new());
        entity_tl.add_child(entity_bl);

        let entity_br = Entity::new_with_transform(
            Rc::downgrade(&entity_tl),
            "Bottom Right Rotating Sphere".to_owned(),
            Transform {
                position: Vec3::new(3.0, -3.0, 0.0),
                rotation: Quaternion::identity(),
                scale: Vec3::one(),
            },
        );
        entity_br.add_component(RendererComponent::new(model_silver));
        entity_tl.add_child(entity_br);

        scene.add_entity(entity_tl);
    }

    scene.load();

    // setup lights
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

#[derive(Debug)]
struct RotateComponent {
    scene: RefCell<Weak<Scene>>,
    entity: RefCell<Weak<Entity>>,
}

impl RotateComponent {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            scene: Weak::new().into(),
            entity: Weak::new().into(),
        })
    }
}

impl Component for RotateComponent {
    fn attach(
        &self,
        scene: std::rc::Weak<vulkan_engine::scene::Scene>,
        entity: std::rc::Weak<Entity>,
    ) {
        *self.scene.borrow_mut() = scene;
        *self.entity.borrow_mut() = entity;
    }

    fn load(&self) {}

    fn start(&self) {}

    fn update(&self, delta: f32) {
        let entity = self.entity.borrow();
        if let Some(entity) = entity.upgrade() {
            let mut transform = entity.transform.borrow_mut();

            let mut rotation = transform.rotation;
            rotation = rotation
                * Quaternion::axis_angle(Vec3::new(0.0, 0.0, 1.0), 120.0f32.to_radians() * delta);
            transform.rotation = rotation;
        }
    }
}

#[derive(Debug)]
struct ScaleComponent {
    scene: RefCell<Weak<Scene>>,
    entity: RefCell<Weak<Entity>>,
    total_time: Cell<f32>,
}

impl ScaleComponent {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            scene: Weak::new().into(),
            entity: Weak::new().into(),
            total_time: 0.0f32.into(),
        })
    }
}

impl Component for ScaleComponent {
    fn attach(
        &self,
        scene: std::rc::Weak<vulkan_engine::scene::Scene>,
        entity: std::rc::Weak<Entity>,
    ) {
        *self.scene.borrow_mut() = scene;
        *self.entity.borrow_mut() = entity;
    }

    fn load(&self) {}

    fn start(&self) {}

    fn update(&self, delta: f32) {
        let entity = self.entity.borrow();
        if let Some(entity) = entity.upgrade() {
            let time = self.total_time.get() + delta;
            self.total_time.set(time);

            let mut transform = entity.transform.borrow_mut();
            transform.scale = Vec3::one() * ((time * 0.5).sin() + 1.0);
        }
    }
}
