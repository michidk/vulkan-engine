use std::{
    cell::Cell,
    path::Path,
    rc::{Rc, Weak},
};

use gfx_maths::*;
use vulkan_engine::{
    core::{engine::Engine, input::Input},
    scene::{
        component::{
            camera_component::CameraComponent, debug_movement_component::DebugMovementComponent,
            light_component::LightComponent, renderer::RendererComponent, Component,
        },
        entity::Entity,
        light::DirectionalLight,
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
    vulkan_engine::run_engine(1920, 1080, "Components Example", setup);
}

fn setup(engine: &mut Engine) {
    engine.register_component::<RotateComponent>("RotateComponent".to_string());
    engine.register_component::<ScaleComponent>("ScaleComponent".to_string());

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

    let entity_cam = scene.new_entity_with_transform(
        "Main Camera".to_owned(),
        Transform {
            position: Vec3::new(0.0, 0.0, -5.0),
            rotation: Quaternion::identity(),
            scale: Vec3::one(),
        },
    );
    entity_cam.new_component::<CameraComponent>();
    entity_cam.new_component::<DebugMovementComponent>();

    {
        let entity_tl = scene.new_entity_with_transform(
            "Top Left Rotating Sphere".to_owned(),
            Transform {
                position: Vec3::new(0.0, 0.0, 10.0),
                rotation: Quaternion::identity(),
                scale: Vec3::one(),
            },
        );
        *entity_tl
            .new_component::<RendererComponent>()
            .model
            .borrow_mut() = Some(model_red);
        entity_tl.new_component::<RotateComponent>();

        let entity_tr = scene.new_entity_with_transform(
            "Top Right Rotating Sphere".to_owned(),
            Transform {
                position: Vec3::new(3.0, 0.0, 0.0),
                rotation: Quaternion::identity(),
                scale: Vec3::one(),
            },
        );
        *entity_tr
            .new_component::<RendererComponent>()
            .model
            .borrow_mut() = Some(model_silver.clone());
        entity_tr.new_component::<ScaleComponent>();
        entity_tr.attach_to(&entity_tl);

        let entity_bl = scene.new_entity_with_transform(
            "Bottom Left Rotating Sphere".to_owned(),
            Transform {
                position: Vec3::new(0.0, -3.0, 0.0),
                rotation: Quaternion::identity(),
                scale: Vec3::one(),
            },
        );
        *entity_bl
            .new_component::<RendererComponent>()
            .model
            .borrow_mut() = Some(model_silver.clone());
        entity_bl.new_component::<ScaleComponent>();
        entity_bl.attach_to(&entity_tl);

        let entity_br = scene.new_entity_with_transform(
            "Bottom Right Rotating Sphere".to_owned(),
            Transform {
                position: Vec3::new(3.0, -3.0, 0.0),
                rotation: Quaternion::identity(),
                scale: Vec3::one(),
            },
        );
        *entity_br
            .new_component::<RendererComponent>()
            .model
            .borrow_mut() = Some(model_silver);
        entity_br.attach_to(&entity_tl);
    }

    let dirlight1 = scene.new_entity_with_transform(
        "DirLight1".to_string(),
        Transform {
            position: Vec3::zero(),
            rotation: Quaternion::axis_angle(Vec3::new(1.0, 0.0, 0.0), -90.0f32.to_radians()),
            scale: Vec3::one(),
        },
    );
    dirlight1.new_component::<LightComponent>().light.set(
        DirectionalLight {
            direction: Vec4::zero(),
            illuminance: Vec4::new(10.1, 10.1, 10.1, 0.0),
        }
        .into(),
    );
    dirlight1.new_component::<RotateComponent>();

    scene.load();
}

#[derive(Debug)]
struct RotateComponent {
    entity: Weak<Entity>,
    rotation_speed: Cell<f32>,
}

impl Component for RotateComponent {
    fn create(entity: &Rc<Entity>) -> Rc<Self>
    where
        Self: Sized,
    {
        Rc::new(Self {
            entity: Rc::downgrade(entity),
            rotation_speed: Cell::new(120.0),
        })
    }

    fn load(&self) {}

    fn start(&self) {}

    fn update(&self, _input: &Input, delta: f32) {
        if let Some(entity) = self.entity.upgrade() {
            let mut transform = entity.transform.borrow_mut();

            let mut rotation = transform.rotation;
            rotation = Quaternion::axis_angle(
                Vec3::new(0.0, 0.0, 1.0),
                self.rotation_speed.get().to_radians() * delta,
            ) * rotation;
            transform.rotation = rotation;
        }
    }

    fn inspector_name(&self) -> &'static str {
        "RotateComponent"
    }

    fn render_inspector(&self, ui: &imgui::Ui) {
        let mut rot_speed = self.rotation_speed.get();
        ui.slider("Rotation Speed", -360.0, 360.0, &mut rot_speed);
        self.rotation_speed.set(rot_speed);
    }
}

#[derive(Debug)]
struct ScaleComponent {
    entity: Weak<Entity>,
    total_time: Cell<f32>,
    time_scale: Cell<f32>,
    max_scale: Cell<f32>,
}

impl Component for ScaleComponent {
    fn create(entity: &Rc<Entity>) -> Rc<Self>
    where
        Self: Sized,
    {
        Rc::new(Self {
            entity: Rc::downgrade(entity),
            total_time: 0.0f32.into(),
            time_scale: Cell::new(0.5),
            max_scale: Cell::new(2.0),
        })
    }

    fn load(&self) {}

    fn start(&self) {}

    fn update(&self, _input: &Input, delta: f32) {
        if let Some(entity) = self.entity.upgrade() {
            let time = self.total_time.get() + delta;
            self.total_time.set(time);

            let mut transform = entity.transform.borrow_mut();
            transform.scale = Vec3::one()
                * ((time * self.time_scale.get()).sin() + 1.0)
                * 0.5
                * self.max_scale.get();
        }
    }

    fn inspector_name(&self) -> &'static str {
        "ScaleComponent"
    }

    fn render_inspector(&self, ui: &imgui::Ui) {
        let mut time_scale = self.time_scale.get();
        ui.slider("Time Scale", 0.0, 5.0, &mut time_scale);
        self.time_scale.set(time_scale);

        let mut max_scale = self.max_scale.get();
        ui.slider("Max Scale", 0.0, 5.0, &mut max_scale);
        self.max_scale.set(max_scale);
    }
}
