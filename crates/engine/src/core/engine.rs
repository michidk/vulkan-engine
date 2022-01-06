use std::{cell::RefCell, rc::Rc, time::Instant};

use egui::{CollapsingHeader, Color32, RichText, ScrollArea};

use crate::{
    core::{gameloop::GameLoop, input::Input, window},
    scene::{component::Component, entity::Entity, Scene},
    vulkan::VulkanManager,
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

impl EngineInit {
    pub fn new(info: EngineInfo) -> Result<Self, Box<dyn std::error::Error>> {
        let scene = Scene::new();
        let eventloop = winit::event_loop::EventLoop::new();
        let window = info.window_info.build(&eventloop)?;

        let vulkan_manager = VulkanManager::new(info, &window.winit_window, 3)?;
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

                ui_vertex_count: 0,
                ui_index_count: 0,
                ui_mesh_count: 0,

                scene_graph_visible: false,
            },
        })
    }

    pub fn start(self) -> ! {
        window::start(self);
    }
}

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

    ui_vertex_count: u32,
    ui_index_count: u32,
    ui_mesh_count: u32,

    scene_graph_visible: bool,
}

impl Engine {
    pub fn init(&self) {
        self.gameloop.init();
    }

    fn render_component(ui: &mut egui::Ui, component: &dyn Component) {
        ui.label(component.inspector_name());
    }

    fn render_entity(ui: &mut egui::Ui, entity: &Entity) {
        ui.collapsing(
            RichText::new(format!("{} - (ID {})", entity.name, entity.id)).strong(),
            |ui| {
                let components = entity.components.borrow();
                for comp in &*components {
                    Self::render_component(ui, &**comp);
                }

                let children = entity.children.borrow();
                for child in &*children {
                    Self::render_entity(ui, child);
                }
            },
        );
    }

    pub(crate) fn render(&mut self) {
        self.fps_count += 1;
        if self.fps_time.elapsed().as_secs() >= 1 {
            self.fps = self.fps_count;
            self.fps_count = 0;
            self.fps_time = Instant::now();
        }

        let frame_time = self.last_frame.elapsed().as_secs_f64();
        self.last_frame = Instant::now();

        let gui_input = self.gui_state.take_egui_input(&self.window.winit_window);
        let (output, shapes) = self.gui_context.run(gui_input, |ctx| {
            egui::Window::new("Debug Tools")
                .title_bar(true)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    let fps_color = match self.fps {
                        0..=29 => Color32::RED,
                        30..=59 => Color32::YELLOW,
                        _ => Color32::WHITE,
                    };
                    ui.colored_label(fps_color, format!("FPS: {}", self.fps));
                    ui.colored_label(
                        fps_color,
                        format!("Frame time: {:.3} ms", frame_time * 1000.0),
                    );

                    ui.checkbox(&mut self.vulkan_manager.enable_wireframe, "Wireframe");

                    ui.checkbox(&mut self.scene_graph_visible, "Scene graph");

                    CollapsingHeader::new("UI Debugging").show(ui, |ui| {
                        ui.checkbox(&mut self.vulkan_manager.enable_ui_wireframe, "UI Wireframe");

                        ui.label(format!("Vertices: {}", self.ui_vertex_count));
                        ui.label(format!(
                            "Indices: {} ({} triangles)",
                            self.ui_index_count,
                            self.ui_index_count / 3
                        ));
                        ui.label(format!("Draw calls: {}", self.ui_mesh_count));
                    });
                });

            let root_entity = self.scene.root_entity.borrow();
            if self.scene_graph_visible {
                egui::SidePanel::right("Scene graph")
                    .resizable(true)
                    .show(ctx, |ui| {
                        ScrollArea::vertical().show(ui, |ui| {
                            Self::render_entity(ui, &root_entity);
                        });
                    });
            }
        });
        self.gui_state
            .handle_output(&self.window.winit_window, &self.gui_context, output);
        let gui_meshes = self.gui_context.tessellate(shapes);

        self.ui_vertex_count = gui_meshes.iter().map(|m| m.1.vertices.len() as u32).sum();
        self.ui_index_count = gui_meshes.iter().map(|m| m.1.indices.len() as u32).sum();
        self.ui_mesh_count = gui_meshes.len() as u32;

        let vk = &mut self.vulkan_manager;

        // prepare for render
        let image_index = vk.next_frame();
        vk.wait_for_fence();
        vk.upload_ui_data(self.gui_context.clone(), gui_meshes);
        vk.wait_for_uploads();

        vk.update_commandbuffer(image_index as usize, Rc::clone(&self.scene))
            .expect("updating the command buffer");

        // finanlize renderpass
        vk.submit();
        vk.present(image_index);
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.vulkan_manager.wait_idle();
    }
}
