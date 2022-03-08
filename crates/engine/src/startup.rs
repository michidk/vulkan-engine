use std::rc::Rc;

use env_logger::Env;
use winit::event::WindowEvent;

use crate::{version::Version, graphics::{context::Context, renderer::{deferred::DeferredRenderer, Renderer}, window::Window}};


pub struct AppConfig {
    pub app_info: AppInfo,
    pub engine_config: EngineConfig,
}

pub struct AppInfo {
    pub app_name: &'static str,
    pub app_version: Version,
}

pub struct EngineConfig {
    pub window_width: u32,
    pub window_height: u32,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self { 
            window_width: 800, 
            window_height: 600,
        }
    }
}

pub fn run(app_config: AppConfig) -> ! {
    #[cfg(debug_assertions)]
    let level = "debug";
    #[cfg(not(debug_assertions))]
    let level = "warn";
    env_logger::init_from_env(Env::default().default_filter_or(level));

    let context = Rc::new(Context::new(&app_config.app_info).expect("Failed to create Graphics Context"));
    let renderer = DeferredRenderer::create(Rc::clone(&context)).expect("Failed to create Renderer");

    let event_loop = winit::event_loop::EventLoop::new();
    let main_window = Window::new(800, 600, app_config.app_info.app_name, true, true, Rc::clone(&context), &event_loop).expect("Failed to create main window");

    event_loop.run(move |event, window_target, control_flow| {
        *control_flow = winit::event_loop::ControlFlow::Poll;

        match event {
            winit::event::Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = winit::event_loop::ControlFlow::Exit;
            },
            winit::event::Event::MainEventsCleared => {
                
            },
            winit::event::Event::LoopDestroyed => {
                context.device_wait_idle();
            },
            _ => {},
        }
    });
}
