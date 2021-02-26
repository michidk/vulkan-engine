use winit::event::{Event, WindowEvent};

use crate::engine::EngineInit;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Info {
    pub initial_dimensions: Dimensions,
    pub title: &'static str,
}

impl Info {
    pub fn into_window<T: 'static>(
        self,
        window_target: &winit::event_loop::EventLoopWindowTarget<T>,
    ) -> Result<winit::window::Window, winit::error::OsError> {
        winit::window::WindowBuilder::new()
            .with_title(self.title)
            .with_inner_size(winit::dpi::LogicalSize::new(
                f64::from(self.initial_dimensions.width),
                f64::from(self.initial_dimensions.height),
            ))
            .build(window_target)
    }
}

pub fn start(engine_init: EngineInit) -> ! {
    let mut engine = engine_init.engine;
    engine_init.eventloop.run(move |event, _, controlflow| {
        *controlflow = winit::event_loop::ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *controlflow = winit::event_loop::ControlFlow::Exit,
            #[allow(unused_variables)]
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {}
            #[allow(unused_variables)]
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                state,
                                virtual_keycode: Some(keycode),
                                ..
                            },
                        ..
                    },
                ..
            } => match keycode {
                winit::event::VirtualKeyCode::Up | winit::event::VirtualKeyCode::W => {
                    // fwd = state == winit::event::ElementState::Pressed;
                }
                winit::event::VirtualKeyCode::Down | winit::event::VirtualKeyCode::S => {
                    // back = state == winit::event::ElementState::Pressed;
                }
                winit::event::VirtualKeyCode::A | winit::event::VirtualKeyCode::Left => {
                    // left = state == winit::event::ElementState::Pressed;
                }
                winit::event::VirtualKeyCode::D | winit::event::VirtualKeyCode::Right => {
                    // right = state == winit::event::ElementState::Pressed;
                }
                winit::event::VirtualKeyCode::R => {
                    // renderer.recreate_swapchain().expect("swapchain recreation");
                }
                winit::event::VirtualKeyCode::F12 => {
                    // renderer::screenshot(&renderer).expect("screenshot trouble");
                }
                winit::event::VirtualKeyCode::Q => {
                    *controlflow = winit::event_loop::ControlFlow::Exit;
                }
                _ => {}
            },
            Event::MainEventsCleared => {}
            Event::RedrawRequested(_) => {
                println!("test");
                engine
                    .gameloop
                    .update(&mut engine.vulkan_manager, &engine.scene);
            }
            _ => {}
        }
    });
}
