use std::rc::Rc;

use raw_window_handle::HasRawDisplayHandle;
use ve_renderer::{
    renderer::{AppInfo, Renderer},
    version::Version, window::Window,
};
use winit::{event_loop::EventLoop, platform::run_return::EventLoopExtRunReturn};

fn main() {
    let event_loop = EventLoop::new();

    let renderer = Rc::new(Renderer::new(
        &AppInfo {
            name: "Basic Test App",
            version: Version::new(0, 1, 0),
        },
        event_loop.raw_display_handle(),
    )
    .expect("Failed to initialize Renderer"));

    let window = winit::window::WindowBuilder::new()
        .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
        .with_visible(true)
        .with_resizable(true)
        .with_title("TestWindow")
        .build(&event_loop)
        .expect("Failed to create window");
    let window = Window::new(renderer.clone(), window).expect("Failed to create Renderer Window");

    let window2 = winit::window::WindowBuilder::new()
        .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
        .with_visible(true)
        .with_resizable(true)
        .with_title("TestWindow")
        .build(&event_loop)
        .expect("Failed to create window");
    let window2 = Window::new(renderer, window2).expect("Failed to create Renderer Window");

    event_loop.run(|a, _, c| {
        
    });
}
