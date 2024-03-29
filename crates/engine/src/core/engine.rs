use std::{
    cell::RefCell,
    ffi::CStr,
    rc::Rc,
    time::{Duration, Instant},
};

use ash::vk;
use egui::{
    plot::{Legend, Line, Plot, Value, Values},
    CollapsingHeader, Color32, ComboBox, CtxRef, DragValue, ProgressBar, RichText, ScrollArea,
    SidePanel,
};
use gfx_maths::Quaternion;
use serde::{Deserialize, Serialize};

use crate::{
    core::{gameloop::GameLoop, input::Input, window},
    scene::{
        component::{
            camera_component::CameraComponent, debug_movement_component::DebugMovementComponent,
            light_component::LightComponent, renderer::RendererComponent, Component,
        },
        entity::Entity,
        Scene,
    },
    vulkan::{self, RendererConfig, VulkanManager},
};

use super::window::Window;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct EngineInfo {
    pub window_info: window::InitialWindowInfo,
    pub app_name: &'static str,
}

pub struct EngineInit {
    pub eventloop: winit::event_loop::EventLoop<()>,
    pub engine: Engine,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub(crate) struct EngineConfig {
    pub(crate) renderer: Option<vulkan::RendererConfig>,
}

pub(crate) fn read_config() -> EngineConfig {
    if let Ok(content) = std::fs::read_to_string("engine.toml") {
        let config: EngineConfig = toml::from_str(&content).unwrap_or_default();
        config
    } else {
        EngineConfig::default()
    }
}

pub(crate) fn write_config(config: &EngineConfig) {
    if let Ok(content) = toml::to_string_pretty(config) {
        let _ = std::fs::write("engine.toml", content);
    }
}

impl EngineInit {
    pub fn new(info: EngineInfo) -> Result<Self, Box<dyn std::error::Error>> {
        let config = read_config();

        let scene = Scene::new();
        let eventloop = winit::event_loop::EventLoop::new();
        let window = info.window_info.build(&eventloop)?;

        let vulkan_manager =
            VulkanManager::new(info, &window.winit_window, 3, config.renderer.as_ref())?;
        let input = Rc::new(RefCell::new(Input::new()));
        let gameloop = GameLoop::new(input.clone());

        let gui_context = egui::CtxRef::default();
        let gui_state = egui_winit::State::new(&window.winit_window);

        Ok(Self {
            eventloop,
            engine: Engine {
                info,
                gameloop,
                input,
                scene,
                vulkan_manager,
                window,
                gui_context,
                gui_state,

                fps_time: Instant::now(),
                fps_count: 0,
                fps: 0,
                last_frame: Instant::now(),
                frame_time_last_sample: Instant::now(),
                frame_time_max: 0.0,
                render_time_max: 0.0,
                ui_time_max: 0.0,
                last_render_time: 0.0,
                last_ui_time: 0.0,
                frame_time_history: Vec::with_capacity(5000),
                render_time_history: Vec::with_capacity(5000),
                ui_time_history: Vec::with_capacity(5000),

                ui_vertex_count: 0,
                ui_index_count: 0,
                ui_mesh_count: 0,

                scene_graph_visible: false,
                #[cfg(feature = "profiler")]
                profiler_visible: false,
                config: read_config(),

                component_factories: Vec::new(),
                selected_entity: None,
                selected_factory: 0,
            },
        })
    }

    pub fn start(mut self) -> ! {
        self.engine
            .register_component::<RendererComponent>("RendererComponent".to_string());
        self.engine
            .register_component::<LightComponent>("LightComponent".to_string());
        self.engine
            .register_component::<DebugMovementComponent>("DebugMovementComponent".to_string());
        self.engine
            .register_component::<CameraComponent>("CameraComponent".to_string());

        window::start(self);
    }
}

pub type ComponentFactoryFn = fn(&Rc<Entity>) -> Rc<dyn Component>;

pub struct Engine {
    pub info: EngineInfo,
    pub(crate) gameloop: GameLoop,
    pub input: Rc<RefCell<Input>>,
    pub scene: Rc<Scene>,
    pub vulkan_manager: VulkanManager,
    pub window: Window,
    pub gui_context: egui::CtxRef,
    pub gui_state: egui_winit::State,

    fps_time: Instant,
    fps_count: usize,
    fps: usize,
    last_frame: Instant,
    frame_time_last_sample: Instant,
    frame_time_max: f32,
    render_time_max: f32,
    ui_time_max: f32,
    last_render_time: f32,
    last_ui_time: f32,
    frame_time_history: Vec<Value>,
    render_time_history: Vec<Value>,
    ui_time_history: Vec<Value>,

    ui_vertex_count: u32,
    ui_index_count: u32,
    ui_mesh_count: u32,

    scene_graph_visible: bool,
    #[cfg(feature = "profiler")]
    profiler_visible: bool,
    config: EngineConfig,

    component_factories: Vec<(String, ComponentFactoryFn)>,
    selected_entity: Option<Rc<Entity>>,
    selected_factory: usize,
}

impl Engine {
    pub fn init(&self) {
        self.gameloop.init();
    }

    pub fn register_component<T: Component + 'static>(&mut self, name: String) {
        self.component_factories.push((name, |e| T::create(e)));
    }

    fn render_component(ui: &mut egui::Ui, component: &dyn Component) {
        ui.label(component.inspector_name());
        ui.indent("", |ui| {
            component.render_inspector(ui);
        });
    }

    fn render_inspector(
        ctx: &CtxRef,
        selected_entity: &mut Option<Rc<Entity>>,
        selected_factory: &mut usize,
        factories: &[(String, ComponentFactoryFn)],
    ) {
        SidePanel::right("inspector")
            .resizable(true)
            .show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    if let Some(entity) = selected_entity {
                        ui.heading(format!("Entity {} ({}) selected", entity.id, &entity.name));

                        if ui.button("Deselect").clicked() {
                            *selected_entity = None;
                            return;
                        }

                        let mut transform = entity.transform.borrow_mut();

                        ui.label("Position");
                        ui.horizontal(|ui| {
                            ui.add(DragValue::new(&mut transform.position.x).prefix("X: "));
                            ui.add(DragValue::new(&mut transform.position.y).prefix("Y: "));
                            ui.add(DragValue::new(&mut transform.position.z).prefix("Z: "));
                        });

                        ui.label("Rotation");
                        ui.horizontal(|ui| {
                            let mut euler = transform.rotation.to_euler_angles_zyx();

                            if ui
                                .add(
                                    DragValue::new(&mut euler.x)
                                        .prefix("Pitch: ")
                                        .max_decimals(2),
                                )
                                .changed()
                                || ui
                                    .add(
                                        DragValue::new(&mut euler.y)
                                            .prefix("Yaw: ")
                                            .max_decimals(2),
                                    )
                                    .changed()
                                || ui
                                    .add(
                                        DragValue::new(&mut euler.z)
                                            .prefix("Roll: ")
                                            .max_decimals(2),
                                    )
                                    .changed()
                            {
                                transform.rotation = Quaternion::from_euler_angles_zyx(&euler);
                            }
                        });

                        ui.label("Scale");
                        ui.horizontal(|ui| {
                            ui.add(DragValue::new(&mut transform.scale.x).prefix("X: "));
                            ui.add(DragValue::new(&mut transform.scale.y).prefix("Y: "));
                            ui.add(DragValue::new(&mut transform.scale.z).prefix("Z: "));
                        });

                        ui.separator();

                        ui.heading("Components");

                        for comp in &*entity.components.borrow() {
                            Self::render_component(ui, &**comp);
                        }

                        ui.horizontal(|ui| {
                            ComboBox::from_id_source("factory_selection").show_index(
                                ui,
                                selected_factory,
                                factories.len(),
                                |i| factories[i].0.clone(),
                            );

                            if ui.button("Add").clicked() {
                                entity.new_component_with_factory(factories[*selected_factory].1);
                            }
                        });
                    } else {
                        ui.heading("Inspector");
                    }
                });
                ui.allocate_space(ui.available_size());
            });
    }

    fn render_entity(
        ui: &mut egui::Ui,
        entity: &Rc<Entity>,
        selected_entity: &mut Option<Rc<Entity>>,
    ) {
        ui.collapsing(
            RichText::new(format!("{} - (ID {})", entity.name, entity.id)).strong(),
            |ui| {
                if ui.button("Inspect").clicked() {
                    *selected_entity = Some(entity.clone());
                }

                if ui.button("Add child").clicked() {
                    entity.add_new_child("New Entity".to_string());
                }

                let children = entity.children.borrow();
                for child in &*children {
                    Self::render_entity(ui, child, selected_entity);
                }
            },
        );
    }

    pub(crate) fn render(&mut self) {
        profile_function!();

        let frame_time = self.update_frame_stats();

        let gui_meshes = self.render_debug_ui(frame_time);

        self.render_3d(gui_meshes);
    }

    fn update_frame_stats(&mut self) -> f32 {
        self.fps_count += 1;
        if self.fps_time.elapsed().as_secs() >= 1 {
            self.fps = self.fps_count;
            self.fps_count = 0;
            self.fps_time = Instant::now();
        }

        let frame_time = self.last_frame.elapsed().as_secs_f32() * 1000.0;
        self.last_frame = Instant::now();

        if frame_time > self.frame_time_max {
            self.frame_time_max = frame_time;
            self.render_time_max = self.last_render_time;
            self.ui_time_max = self.last_ui_time;
        }

        if self.frame_time_last_sample.elapsed().as_millis() >= 100 {
            let plot_x = self
                .frame_time_history
                .last()
                .map(|v| v.x + 0.1)
                .unwrap_or(0.0);

            self.frame_time_history.push(Value::new(
                plot_x,
                if plot_x == 0.0 {
                    0.0
                } else {
                    self.frame_time_max
                },
            ));
            if self.frame_time_history.len() > 10 * 10 {
                self.frame_time_history.remove(0);
            }

            self.render_time_history.push(Value::new(
                plot_x,
                if plot_x == 0.0 {
                    0.0
                } else {
                    self.render_time_max
                },
            ));
            if self.render_time_history.len() > 10 * 10 {
                self.render_time_history.remove(0);
            }

            self.ui_time_history.push(Value::new(
                plot_x,
                if plot_x == 0.0 {
                    0.0
                } else {
                    self.ui_time_max + self.render_time_max
                },
            ));
            if self.ui_time_history.len() > 10 * 10 {
                self.ui_time_history.remove(0);
            }

            self.frame_time_last_sample += Duration::from_millis(100);

            self.frame_time_max = 0.0;
            self.render_time_max = 0.0;
            self.ui_time_max = 0.0;
        }

        frame_time
    }

    fn render_3d(&mut self, gui_meshes: Vec<egui::ClippedMesh>) {
        profile_function!();

        let render_start_time = Instant::now();

        let vk = &mut self.vulkan_manager;

        {
            // prepare for render
            let image_index = vk.next_frame();
            vk.wait_for_fence();
            vk.upload_ui_data(self.gui_context.clone(), gui_meshes);
            vk.wait_for_uploads();

            vk.update_commandbuffer(image_index as usize, Rc::clone(&self.scene))
                .expect("updating the command buffer");

            // finalize renderpass
            vk.submit();
            vk.present(image_index);
        }

        let render_time = render_start_time.elapsed().as_secs_f32() * 1000.0;
        self.last_render_time = render_time;
    }

    fn render_debug_tools_window(&mut self, ctx: &egui::CtxRef, frame_time: f32) {
        profile_function!();

        egui::Window::new("Debug Tools")
            .title_bar(true)
            .collapsible(false)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Frame timing");

                let fps_color = match self.fps {
                    0..=29 => Color32::RED,
                    30..=59 => Color32::YELLOW,
                    _ => Color32::WHITE,
                };
                ui.colored_label(fps_color, format!("FPS: {}", self.fps));
                ui.colored_label(fps_color, format!("Frame time: {frame_time:.3} ms"));

                Plot::new("Frame time Graph")
                    .height(70.0)
                    .include_y(0.0)
                    .legend(Legend::default())
                    .show(ui, |ui| {
                        let line = Line::new(Values::from_values(self.frame_time_history.clone()))
                            .name("Frame")
                            .fill(0.0)
                            .highlight();
                        ui.line(line);

                        let line = Line::new(Values::from_values(self.render_time_history.clone()))
                            .name("Render")
                            .fill(0.0)
                            .highlight();
                        ui.line(line);

                        let line = Line::new(Values::from_values(self.ui_time_history.clone()))
                            .name("UI")
                            .fill(0.0)
                            .highlight();
                        ui.line(line);
                    });

                #[cfg(feature = "profiler")]
                if ui.button("Profiler").clicked() {
                    self.profiler_visible = true;
                    puffin::set_scopes_on(true);
                }

                if let Some((budget, heap_count)) = self.vulkan_manager.get_budget() {
                    ui.separator();
                    ui.heading("Memory usage");

                    for (i, (available, used)) in budget.heap_budget[..heap_count]
                        .iter()
                        .zip(&budget.heap_usage[..heap_count])
                        .enumerate()
                    {
                        let portion = *used as f32 / *available as f32;

                        let budget_mb = (*available as f32) / 1024.0 / 1024.0;
                        let used_mb = (*used as f32) / 1024.0 / 1024.0;

                        ui.scope(|ui| {
                            if portion > 0.8 {
                                ui.style_mut().visuals.selection.bg_fill = Color32::RED;
                            } else if portion > 0.6 {
                                ui.style_mut().visuals.selection.bg_fill =
                                    Color32::from_rgb(255, 127, 0);
                            }

                            ui.add(ProgressBar::new(portion).text(format!(
                                "Heap {}: {:.3} MB/{:.3} MB ({:.2}%)",
                                i,
                                used_mb,
                                budget_mb,
                                portion * 100.0
                            )));
                        });
                    }
                }

                ui.separator();
                ui.heading("Debugging");

                ui.checkbox(&mut self.vulkan_manager.enable_wireframe, "Wireframe");

                ui.checkbox(&mut self.scene_graph_visible, "Show scene graph");

                CollapsingHeader::new("UI Debugging").show(ui, |ui| {
                    ui.checkbox(&mut self.vulkan_manager.enable_ui_wireframe, "UI Triangles");

                    ui.label(format!("Vertices: {}", self.ui_vertex_count));
                    ui.label(format!(
                        "Indices: {} ({} triangles)",
                        self.ui_index_count,
                        self.ui_index_count / 3
                    ));
                    ui.label(format!("Draw calls: {}", self.ui_mesh_count));
                });

                ui.separator();
                ui.heading("GPU Override");

                ui.label(format!(
                    "Current device: {} ({:08X}:{:08X}) Vulkan {}.{}.{}",
                    unsafe {
                        CStr::from_ptr(
                            self.vulkan_manager
                                .physical_device_properties
                                .device_name
                                .as_ptr(),
                        )
                    }
                    .to_str()
                    .unwrap(),
                    self.vulkan_manager.physical_device_properties.vendor_id,
                    self.vulkan_manager.physical_device_properties.device_id,
                    vk::api_version_major(
                        self.vulkan_manager.physical_device_properties.api_version
                    ),
                    vk::api_version_minor(
                        self.vulkan_manager.physical_device_properties.api_version
                    ),
                    vk::api_version_patch(
                        self.vulkan_manager.physical_device_properties.api_version
                    ),
                ));

                let device_override = self
                    .config
                    .renderer
                    .as_ref()
                    .map(|rc| rc.gpu_vendor_id.zip(rc.gpu_device_id))
                    .unwrap_or_default();

                if ui
                    .selectable_label(device_override.is_none(), "Default")
                    .clicked()
                {
                    log::info!("Clearing GPU override");
                    self.config.renderer = None;
                    write_config(&self.config);

                    let args = std::env::args().collect::<Vec<_>>();
                    std::process::Command::new(&args[0])
                        .args(args)
                        .spawn()
                        .unwrap();
                    std::process::exit(0);
                }
                for (_, props, _) in &self.vulkan_manager.supported_devices {
                    let name = unsafe { CStr::from_ptr(props.device_name.as_ptr()) }
                        .to_str()
                        .unwrap();

                    let clicked = ui
                        .selectable_label(
                            device_override.map_or(false, |d| {
                                d.0 == props.vendor_id && d.1 == props.device_id
                            }),
                            format!(
                                "{} ({:08X}:{:08X}) Vulkan {}.{}.{}",
                                name,
                                props.vendor_id,
                                props.device_id,
                                vk::api_version_major(props.api_version),
                                vk::api_version_minor(props.api_version),
                                vk::api_version_patch(props.api_version),
                            ),
                        )
                        .clicked();
                    if clicked {
                        log::info!("Overriding config with new device");
                        self.config.renderer = Some(RendererConfig {
                            gpu_vendor_id: Some(props.vendor_id),
                            gpu_device_id: Some(props.device_id),
                        });
                        write_config(&self.config);

                        let args = std::env::args().collect::<Vec<_>>();
                        std::process::Command::new(&args[0])
                            .args(args)
                            .spawn()
                            .unwrap();
                        std::process::exit(0);
                    }
                }
            });
    }

    fn render_scene_graph(&mut self, ctx: &egui::CtxRef) {
        profile_function!();

        let root_entity = self.scene.root_entity.borrow();
        if self.scene_graph_visible {
            egui::SidePanel::right("Scene graph")
                .resizable(true)
                .show(ctx, |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        Self::render_entity(ui, &root_entity, &mut self.selected_entity);
                    });
                    ui.allocate_space(ui.available_size());
                });
        }
    }

    fn render_debug_ui(&mut self, frame_time: f32) -> Vec<egui::ClippedMesh> {
        profile_function!();

        let ui_start_time = Instant::now();

        let mut gui_context = self.gui_context.clone();

        let gui_input = self.gui_state.take_egui_input(&self.window.winit_window);
        let (output, shapes) = gui_context.run(gui_input, |ctx| {
            self.render_debug_tools_window(ctx, frame_time);

            self.render_scene_graph(ctx);

            if self.scene_graph_visible {
                Self::render_inspector(
                    ctx,
                    &mut self.selected_entity,
                    &mut self.selected_factory,
                    &self.component_factories,
                );
            }

            #[cfg(feature = "profiler")]
            if self.profiler_visible && !puffin_egui::profiler_window(ctx) {
                self.profiler_visible = false;
                puffin::set_scopes_on(false);
            }
        });

        // After every frame, an egui::CtxRef refers to an entirely new object, even though an "xxxRef" sounds like it works just like an Arc<...>.
        // Because of this, we need to reassign self.gui_context to the "new" gui_context.
        self.gui_context = gui_context;

        let gui_meshes;
        {
            profile_scope!("Ui tesselation");
            self.gui_state
                .handle_output(&self.window.winit_window, &self.gui_context, output);
            gui_meshes = self.gui_context.tessellate(shapes);

            self.ui_vertex_count = gui_meshes.iter().map(|m| m.1.vertices.len() as u32).sum();
            self.ui_index_count = gui_meshes.iter().map(|m| m.1.indices.len() as u32).sum();
            self.ui_mesh_count = gui_meshes.len() as u32;
        }

        let ui_time = ui_start_time.elapsed().as_secs_f32() * 1000.0;
        self.last_ui_time = ui_time;

        gui_meshes
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.selected_entity = None;
        self.vulkan_manager.wait_idle();
    }
}
