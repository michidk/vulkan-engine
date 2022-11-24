use std::rc::Rc;

use raw_window_handle::HasRawDisplayHandle;
use ve_renderer::{
    renderer::{AppInfo, Renderer},
    version::Version,
    window::Window,
};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();

    log::info!("Initializing Renderer");
    let renderer = Rc::new(
        Renderer::new(
            &AppInfo {
                name: "Basic Test App",
                version: Version::new(0, 1, 0),
            },
            event_loop.raw_display_handle(),
        )
        .expect("Failed to initialize Renderer"),
    );

    log::info!("Creating Window 1");
    let window = winit::window::WindowBuilder::new()
        .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
        .with_visible(true)
        .with_resizable(true)
        .with_title("TestWindow")
        .build(&event_loop)
        .expect("Failed to create window");
    let wnd_id = window.id();
    let mut window =
        Some(Window::new(renderer.clone(), window).expect("Failed to create Renderer Window"));

    log::info!("Creating Window 1");
    let window2 = winit::window::WindowBuilder::new()
        .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
        .with_visible(true)
        .with_resizable(true)
        .with_title("TestWindow2")
        .build(&event_loop)
        .expect("Failed to create window2");
    let wnd2_id = window2.id();
    let mut window2 =
        Some(Window::new(renderer, window2).expect("Failed to create Renderer Window"));

    log::info!("Running event loop");
    event_loop.run(move |a, _, c| {
        *c = ControlFlow::Poll;

        if let winit::event::Event::WindowEvent {
            window_id,
            event: winit::event::WindowEvent::CloseRequested,
        } = a
        {
            if window_id == wnd_id {
                window = None;
            } else if window_id == wnd2_id {
                window2 = None;
            }

            if window.is_none() && window2.is_none() {
                *c = ControlFlow::Exit;
            }
        }
    });
}
