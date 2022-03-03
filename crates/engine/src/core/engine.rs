use std::{
    cell::RefCell,
    ffi::{CStr, CString},
    rc::Rc,
    time::{Duration, Instant},
};

use ash::vk;

use gfx_maths::{Quaternion, Vec3};
use imgui::{
    Condition, Direction, FontConfig, FontSource, StyleColor, StyleVar, TreeNodeFlags, WindowFlags,
};
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
    pub(crate) ui: Option<UiConfig>,
    pub(crate) renderer: Option<vulkan::RendererConfig>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy)]
pub(crate) struct UiConfig {
    pub(crate) scene_graph_visible: Option<bool>,
    pub(crate) inspector_visible: Option<bool>,
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

        let mut imgui = imgui::Context::create();
        let mut imgui_platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
        imgui_platform.attach_window(
            imgui.io_mut(),
            &window.winit_window,
            imgui_winit_support::HiDpiMode::Default,
        );

        let font_data = std::fs::read("assets/fonts/Roboto/Roboto-Regular.ttf").unwrap();
        let font_data_bold = std::fs::read("assets/fonts/Roboto/Roboto-Bold.ttf").unwrap();

        imgui.fonts().clear();
        let font_normal = imgui.fonts().add_font(&[FontSource::TtfData {
            data: &font_data,
            size_pixels: 14.0,
            config: Some(FontConfig {
                name: Some("Default".to_string()),
                ..Default::default()
            }),
        }]);
        let font_bold = imgui.fonts().add_font(&[FontSource::TtfData {
            data: &font_data_bold,
            size_pixels: 14.0,
            config: Some(FontConfig {
                name: Some("Bold".to_string()),
                ..Default::default()
            }),
        }]);
        let font_heading = imgui.fonts().add_font(&[FontSource::TtfData {
            data: &font_data_bold,
            size_pixels: 16.0,
            config: Some(FontConfig {
                name: Some("Heading".to_string()),
                ..Default::default()
            }),
        }]);

        imgui.fonts().build_rgba32_texture();
        imgui.fonts().clear_input_data();

        imgui.io_mut().config_flags |= imgui::ConfigFlags::DOCKING_ENABLE;
        imgui.style_mut().frame_rounding = 0.0;
        imgui.style_mut().popup_rounding = 0.0;
        imgui.style_mut().scrollbar_rounding = 0.0;
        imgui.style_mut().tab_rounding = 0.0;
        imgui.style_mut().window_rounding = 0.0;
        imgui.style_mut().window_border_size = 0.0;
        imgui.style_mut().popup_border_size = 0.0;
        imgui.style_mut().tab_rounding = 3.0;
        imgui.style_mut().window_menu_button_position = Direction::None;

        imgui.style_mut().colors[StyleColor::WindowBg as usize] = [0.06, 0.06, 0.06, 1.00];
        imgui.style_mut().colors[StyleColor::PopupBg as usize] = [0.08, 0.08, 0.08, 1.00];
        imgui.style_mut().colors[StyleColor::FrameBg as usize] = [0.15, 0.15, 0.15, 0.54];
        imgui.style_mut().colors[StyleColor::FrameBgHovered as usize] = [0.24, 0.24, 0.24, 0.40];
        imgui.style_mut().colors[StyleColor::FrameBgActive as usize] = [0.35, 0.35, 0.35, 0.67];
        imgui.style_mut().colors[StyleColor::TitleBgActive as usize] = [0.04, 0.04, 0.04, 1.00];
        imgui.style_mut().colors[StyleColor::TitleBgCollapsed as usize] = [0.04, 0.04, 0.04, 1.00];
        imgui.style_mut().colors[StyleColor::MenuBarBg as usize] = [0.04, 0.04, 0.04, 1.00];
        imgui.style_mut().colors[StyleColor::CheckMark as usize] = [1.00, 1.00, 1.00, 1.00];
        imgui.style_mut().colors[StyleColor::Header as usize] = [0.00, 0.00, 0.00, 0.30];
        imgui.style_mut().colors[StyleColor::HeaderHovered as usize] = [0.15, 0.15, 0.15, 0.80];
        imgui.style_mut().colors[StyleColor::HeaderActive as usize] = [0.27, 0.27, 0.27, 1.00];
        imgui.style_mut().colors[StyleColor::Tab as usize] = [0.15, 0.15, 0.15, 0.86];
        imgui.style_mut().colors[StyleColor::TabHovered as usize] = [0.27, 0.27, 0.27, 0.80];
        imgui.style_mut().colors[StyleColor::TabActive as usize] = [0.35, 0.35, 0.35, 1.00];
        imgui.style_mut().colors[StyleColor::TabUnfocused as usize] = [0.15, 0.15, 0.15, 0.97];
        imgui.style_mut().colors[StyleColor::TabUnfocusedActive as usize] =
            [0.35, 0.35, 0.35, 1.00];
        imgui.style_mut().colors[StyleColor::SliderGrab as usize] = [0.81, 0.33, 0.00, 1.00];
        imgui.style_mut().colors[StyleColor::SliderGrabActive as usize] = [1.00, 0.41, 0.00, 1.00];
        imgui.style_mut().colors[StyleColor::Button as usize] = [0.15, 0.15, 0.15, 0.40];
        imgui.style_mut().colors[StyleColor::ButtonHovered as usize] = [0.27, 0.27, 0.27, 1.00];
        imgui.style_mut().colors[StyleColor::ButtonActive as usize] = [0.35, 0.35, 0.35, 1.00];
        imgui.style_mut().colors[StyleColor::SeparatorHovered as usize] = [0.81, 0.33, 0.00, 1.00];
        imgui.style_mut().colors[StyleColor::SeparatorActive as usize] = [1.00, 0.41, 0.00, 1.00];
        imgui.style_mut().colors[StyleColor::ResizeGrip as usize] = [0.82, 0.34, 0.00, 0.35];
        imgui.style_mut().colors[StyleColor::ResizeGripHovered as usize] = [0.93, 0.38, 0.00, 0.35];
        imgui.style_mut().colors[StyleColor::ResizeGripActive as usize] = [1.00, 0.41, 0.00, 0.35];
        imgui.style_mut().colors[StyleColor::DockingPreview as usize] = [1.00, 0.41, 0.00, 0.70];
        imgui.style_mut().colors[StyleColor::TextSelectedBg as usize] = [1.00, 0.44, 0.00, 0.35];

        Ok(Self {
            eventloop,
            engine: Engine {
                info,
                gameloop,
                input,
                scene,
                vulkan_manager,
                window,
                imgui,
                imgui_platform,

                ui: EngineUi {
                    ui_vertex_count: 0,
                    ui_index_count: 0,
                    ui_mesh_count: 0,
                    #[cfg(feature = "profiler")]
                    profiler_visible: false,
                    selected_entity: None,
                    font_normal,
                    font_bold,
                    font_heading,
                },

                frame_time_info: FrameTimeInfo {
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
                },

                config: read_config(),

                component_factories: Vec::new(),
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

    pub(crate) imgui: imgui::Context,
    pub(crate) imgui_platform: imgui_winit_support::WinitPlatform,

    frame_time_info: FrameTimeInfo,

    pub(crate) ui: EngineUi,

    config: EngineConfig,

    component_factories: Vec<(String, ComponentFactoryFn)>,
}

pub(crate) struct FrameTimeInfo {
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
    frame_time_history: Vec<f32>,
    render_time_history: Vec<f32>,
    ui_time_history: Vec<f32>,
}

pub(crate) struct EngineUi {
    ui_vertex_count: u32,
    ui_index_count: u32,
    ui_mesh_count: u32,

    #[cfg(feature = "profiler")]
    profiler_visible: bool,

    selected_entity: Option<Rc<Entity>>,

    #[allow(unused)]
    font_normal: imgui::FontId,
    #[allow(unused)]
    font_bold: imgui::FontId,
    font_heading: imgui::FontId,
}

impl Engine {
    pub fn init(&self) {
        self.gameloop.init();
    }

    pub fn register_component<T: Component + 'static>(&mut self, name: String) {
        self.component_factories.push((name, |e| T::create(e)));
    }

    fn render_component(ui: &imgui::Ui, component: &dyn Component) {
        if ui.collapsing_header(component.inspector_name(), TreeNodeFlags::DEFAULT_OPEN) {
            component.render_inspector(ui);
        }
    }

    fn render_inspector(
        ui: &imgui::Ui,
        selected_entity: &mut Option<Rc<Entity>>,
        factories: &[(String, ComponentFactoryFn)],
        heading_font: imgui::FontId,
    ) {
        ui.window("Inspector").build(|| {
            if let Some(entity) = selected_entity {
                let font = ui.push_font(heading_font);
                ui.text(format!("Entity {} ({})", entity.id, &entity.name));
                font.pop();

                let mut transform = entity.transform.borrow_mut();

                let mut pos = [
                    transform.position.x,
                    transform.position.y,
                    transform.position.z,
                ];
                if ui.input_float3("Position", &mut pos).build() {
                    transform.position = Vec3::new(pos[0], pos[1], pos[2]);
                }

                let euler = transform.rotation.to_euler_angles_zyx();
                let mut rot = [euler.x, euler.y, euler.z];
                if ui.input_float3("Rotation", &mut rot).build() {
                    transform.rotation =
                        Quaternion::from_euler_angles_zyx(&Vec3::new(rot[0], rot[1], rot[2]));
                }

                let mut scale = [transform.scale.x, transform.scale.y, transform.scale.z];
                if ui.input_float3("Scale", &mut scale).build() {
                    transform.scale = Vec3::new(scale[0], scale[1], scale[2]);
                }

                ui.separator();

                ui.text("Components");

                for comp in &*entity.components.borrow() {
                    Self::render_component(ui, &**comp);
                }

                if ui.button_with_size("Add Component", [-1.0, 0.0]) {
                    ui.open_popup("Add Component");
                }

                ui.popup("Add Component", || {
                    for (name, factory) in factories {
                        if ui.selectable(name) {
                            entity.new_component_with_factory(factory);
                            ui.close_current_popup();
                        }
                    }
                });
            } else {
                let font = ui.push_font(heading_font);
                ui.text("Inspector");
                font.pop();
            }
        });
    }

    fn render_entity(
        ui: &imgui::Ui,
        entity: &Rc<Entity>,
        selected_entity: &mut Option<Rc<Entity>>,
    ) {
        let mut open_popup = false;

        let node = ui
            .tree_node_config(format!("{} - (ID {})", entity.name, entity.id))
            .leaf(entity.children.borrow().is_empty())
            .open_on_arrow(true)
            .selected(
                selected_entity
                    .as_ref()
                    .map_or(false, |se| se.id == entity.id),
            )
            .push();

        if ui.is_item_clicked() && !ui.is_item_toggled_open() {
            *selected_entity = Some(entity.clone());
        }

        if ui.is_item_clicked_with_button(imgui::MouseButton::Right) {
            open_popup = true;
        }

        if let Some(node) = node {
            let children = entity.children.borrow();
            for child in &*children {
                Self::render_entity(ui, child, selected_entity);
            }

            node.pop();
        }

        let popup_id = ui.push_id_ptr(&*entity);
        if open_popup {
            ui.open_popup("entity-right-click-popup");
        }

        ui.popup("entity-right-click-popup", || {
            if ui.selectable("Add Child") {
                entity.add_new_child("New Entity".to_owned());
                ui.close_current_popup();
            }
        });
        popup_id.pop();
    }

    pub(crate) fn render(&mut self) {
        profile_function!();

        let frame_time = Self::update_frame_stats(&mut self.frame_time_info);

        self.render_debug_ui(frame_time);
    }

    fn update_frame_stats(frame_time_info: &mut FrameTimeInfo) -> f32 {
        frame_time_info.fps_count += 1;
        if frame_time_info.fps_time.elapsed().as_secs() >= 1 {
            frame_time_info.fps = frame_time_info.fps_count;
            frame_time_info.fps_count = 0;
            frame_time_info.fps_time = Instant::now();
        }

        let frame_time = frame_time_info.last_frame.elapsed().as_secs_f32() * 1000.0;
        frame_time_info.last_frame = Instant::now();

        if frame_time > frame_time_info.frame_time_max {
            frame_time_info.frame_time_max = frame_time;
            frame_time_info.render_time_max = frame_time_info.last_render_time;
            frame_time_info.ui_time_max = frame_time_info.last_ui_time;
        }

        if frame_time_info.frame_time_last_sample.elapsed().as_millis() >= 100 {
            frame_time_info
                .frame_time_history
                .push(frame_time_info.frame_time_max);
            if frame_time_info.frame_time_history.len() > 10 * 10 {
                frame_time_info.frame_time_history.remove(0);
            }

            frame_time_info
                .render_time_history
                .push(frame_time_info.render_time_max);
            if frame_time_info.render_time_history.len() > 10 * 10 {
                frame_time_info.render_time_history.remove(0);
            }

            frame_time_info
                .ui_time_history
                .push(frame_time_info.ui_time_max + frame_time_info.render_time_max);
            if frame_time_info.ui_time_history.len() > 10 * 10 {
                frame_time_info.ui_time_history.remove(0);
            }

            frame_time_info.frame_time_last_sample += Duration::from_millis(100);

            frame_time_info.frame_time_max = 0.0;
            frame_time_info.render_time_max = 0.0;
            frame_time_info.ui_time_max = 0.0;
        }

        frame_time
    }

    fn render_debug_tools_window(
        ui: &imgui::Ui,
        engine_ui: &mut EngineUi,
        frame_time_info: &FrameTimeInfo,
        frame_time: f32,
        vulkan_manager: &mut VulkanManager,
        config: &mut EngineConfig,
    ) {
        profile_function!();

        ui.window("Debug Tools").build(|| {
            ui.text("Frame timing");

            let fps_color = match frame_time_info.fps {
                0..=29 => [1.0, 0.0, 0.0, 1.0],
                30..=59 => [1.0, 1.0, 0.0, 1.0],
                _ => [1.0, 1.0, 1.0, 1.0],
            };

            ui.text_colored(fps_color, format!("FPS: {}", frame_time_info.fps));
            ui.text_colored(fps_color, format!("Frame time: {:.3} ms", frame_time));

            ui.plot_lines("Frame time Graph", &frame_time_info.frame_time_history)
                .graph_size([ui.content_region_avail()[0], 100.0])
                .build();

            #[cfg(feature = "profiler")]
            if ui.button("Profiler") {
                engine_ui.profiler_visible = true;
                puffin::set_scopes_on(true);
            }

            if let Some((budget, heap_count)) = vulkan_manager.get_budget() {
                ui.separator();
                ui.text("Memory usage");

                for (i, (available, used)) in budget.heap_budget[..heap_count]
                    .iter()
                    .zip(&budget.heap_usage[..heap_count])
                    .enumerate()
                {
                    let portion = *used as f32 / *available as f32;

                    let budget_mb = (*available as f32) / 1024.0 / 1024.0;
                    let used_mb = (*used as f32) / 1024.0 / 1024.0;

                    imgui::ProgressBar::new(portion)
                        .overlay_text(format!(
                            "Heap {}: {:.3} MB/{:.3} MB ({:.2}%)",
                            i,
                            used_mb,
                            budget_mb,
                            portion * 100.0
                        ))
                        .build(ui);
                }
            }

            ui.separator();

            ui.text("Debugging");

            ui.checkbox("Wireframe", &mut vulkan_manager.enable_wireframe);

            if ui.collapsing_header("UI Debugging", imgui::TreeNodeFlags::FRAMED) {
                ui.checkbox("UI Triangles", &mut vulkan_manager.enable_ui_wireframe);
                ui.text(format!("Vertices: {}", engine_ui.ui_vertex_count));
                ui.text(format!(
                    "Indices: {} ({} triangles)",
                    engine_ui.ui_index_count,
                    engine_ui.ui_index_count / 3
                ));
                ui.text(format!("Draw calls: {}", engine_ui.ui_mesh_count));
            }

            ui.separator();

            ui.text("GPU Override");
            ui.text(format!(
                "Current device: {} ({:08X}:{:08X}) Vulkan {}.{}.{}",
                unsafe {
                    CStr::from_ptr(
                        vulkan_manager
                            .physical_device_properties
                            .device_name
                            .as_ptr(),
                    )
                }
                .to_str()
                .unwrap(),
                vulkan_manager.physical_device_properties.vendor_id,
                vulkan_manager.physical_device_properties.device_id,
                vk::api_version_major(vulkan_manager.physical_device_properties.api_version),
                vk::api_version_minor(vulkan_manager.physical_device_properties.api_version),
                vk::api_version_patch(vulkan_manager.physical_device_properties.api_version),
            ));

            let cfg = config.renderer.unwrap_or_default();
            let dev_override = cfg.gpu_vendor_id.zip(cfg.gpu_device_id);

            if ui
                .selectable_config("Default Device")
                .selected(dev_override.is_none())
                .build()
            {
                log::info!("Clearing GPU override");
                config.renderer = None;
                write_config(config);

                let args = std::env::args().collect::<Vec<_>>();
                std::process::Command::new(&args[0])
                    .args(args)
                    .spawn()
                    .unwrap();
                std::process::exit(0);
            }

            for (_, props, _) in &vulkan_manager.supported_devices {
                let selected = dev_override.map_or(false, |(vendor, dev)| {
                    vendor == props.vendor_id && dev == props.device_id
                });
                if ui
                    .selectable_config(format!(
                        "{} ({:08X}:{:08X}) Vulkan {}.{}.{}",
                        unsafe { CStr::from_ptr(props.device_name.as_ptr()) }
                            .to_str()
                            .unwrap(),
                        props.vendor_id,
                        props.device_id,
                        vk::api_version_major(props.api_version),
                        vk::api_version_minor(props.api_version),
                        vk::api_version_patch(props.api_version),
                    ))
                    .selected(selected)
                    .build()
                {
                    log::info!("Setting GPU override");
                    config.renderer = Some(RendererConfig {
                        gpu_vendor_id: Some(props.vendor_id),
                        gpu_device_id: Some(props.device_id),
                    });
                    write_config(config);

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

    fn render_scene_graph(ui: &imgui::Ui, scene: &Scene, selected_entity: &mut Option<Rc<Entity>>) {
        profile_function!();

        let root_entity = scene.root_entity.borrow();

        ui.window("Scene graph").build(|| {
            Self::render_entity(ui, &root_entity, selected_entity);
        });
    }

    fn render_debug_ui(&mut self, frame_time: f32) {
        profile_function!();

        let ui_start_time = Instant::now();

        let ui = self.imgui.new_frame();

        {
            let padding = ui.push_style_var(StyleVar::WindowPadding([0.0, 0.0]));
            let border_size = ui.push_style_var(StyleVar::WindowBorderSize(0.0));
            let wnd = ui
                .window("Root Window")
                .position([0.0, 0.0], Condition::Always)
                .size(ui.io().display_size, Condition::Always)
                .flags(WindowFlags::NO_DOCKING)
                .menu_bar(true)
                .no_decoration()
                .movable(false)
                .bring_to_front_on_focus(false)
                .nav_focus(false)
                .draw_background(false)
                .begin();
            border_size.pop();
            padding.pop();
            if let Some(wnd) = wnd {
                ui.menu_bar(|| {
                    ui.menu("View", || {
                        if ui
                            .menu_item_config("Scene Graph")
                            .selected(
                                self.config
                                    .ui
                                    .unwrap_or_default()
                                    .scene_graph_visible
                                    .unwrap_or_default(),
                            )
                            .build()
                        {
                            let mut cfg = self.config.ui.unwrap_or_default();
                            cfg.scene_graph_visible =
                                Some(!cfg.scene_graph_visible.unwrap_or_default());
                            self.config.ui = Some(cfg);
                            write_config(&self.config);
                        }
                        if ui
                            .menu_item_config("Inspector")
                            .selected(
                                self.config
                                    .ui
                                    .unwrap_or_default()
                                    .inspector_visible
                                    .unwrap_or_default(),
                            )
                            .build()
                        {
                            let mut cfg = self.config.ui.unwrap_or_default();
                            cfg.inspector_visible =
                                Some(!cfg.inspector_visible.unwrap_or_default());
                            self.config.ui = Some(cfg);
                            write_config(&self.config);
                        }
                    });
                });

                unsafe {
                    let name = CString::new("Root Docking Space").unwrap();
                    let id = imgui_sys::igGetIDStr(name.as_ptr());
                    imgui_sys::igDockSpace(
                        id,
                        imgui_sys::ImVec2 { x: 0.0, y: 0.0 },
                        imgui_sys::ImGuiDockNodeFlags_PassthruCentralNode as i32,
                        std::ptr::null(),
                    );
                }

                ui.window("Style Editor").build(|| {
                    ui.show_default_style_editor();
                });

                let mut opened = true;
                ui.show_demo_window(&mut opened);

                Self::render_debug_tools_window(
                    ui,
                    &mut self.ui,
                    &self.frame_time_info,
                    frame_time,
                    &mut self.vulkan_manager,
                    &mut self.config,
                );

                if self
                    .config
                    .ui
                    .unwrap_or_default()
                    .scene_graph_visible
                    .unwrap_or_default()
                {
                    Self::render_scene_graph(ui, &self.scene, &mut self.ui.selected_entity);
                }

                if self
                    .config
                    .ui
                    .unwrap_or_default()
                    .inspector_visible
                    .unwrap_or_default()
                {
                    Self::render_inspector(
                        ui,
                        &mut self.ui.selected_entity,
                        &self.component_factories,
                        self.ui.font_heading,
                    );
                }

                // #[cfg(feature = "profiler")]
                // if self.profiler_visible && !puffin_imgui:: {
                //     self.profiler_visible = false;
                //     puffin::set_scopes_on(false);
                // }

                wnd.end();
            }
        }

        self.imgui_platform
            .prepare_render(ui, &self.window.winit_window);

        let ui_time = ui_start_time.elapsed().as_secs_f32() * 1000.0;
        self.frame_time_info.last_ui_time = ui_time;

        let render_start_time = Instant::now();

        let vk = &mut self.vulkan_manager;

        {
            // prepare for render
            let image_index = vk.next_frame();
            vk.wait_for_fence();

            vk.upload_font(&mut self.imgui);

            let draw_data;
            {
                profile_scope!("Ui tesselation");
                draw_data = self.imgui.render();
            }

            self.ui.ui_vertex_count = draw_data.total_vtx_count as u32;
            self.ui.ui_index_count = draw_data.total_idx_count as u32;
            self.ui.ui_mesh_count = draw_data.draw_lists_count() as u32;
            vk.upload_ui_data(draw_data);
            vk.wait_for_uploads();

            vk.update_commandbuffer(image_index as usize, Rc::clone(&self.scene))
                .expect("updating the command buffer");

            // finalize renderpass
            vk.submit();
            vk.present(image_index);
        }

        let render_time = render_start_time.elapsed().as_secs_f32() * 1000.0;
        self.frame_time_info.last_render_time = render_time;
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.ui.selected_entity = None;
        self.vulkan_manager.wait_idle();
    }
}
