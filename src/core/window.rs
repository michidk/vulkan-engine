use std::rc::Rc;

use winit::event::{Event, WindowEvent};

use crate::engine::EngineInit;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct WindowInfo {
    pub initial_dimensions: Dimensions,
    pub title: &'static str,
}

impl WindowInfo {
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
            Event::MainEventsCleared => {
                engine
                    .gameloop
                    .update(&mut engine.vulkan_manager, &engine.scene);
                engine.render();
                engine.input.borrow_mut().rollover_state();
            }
            _ => {
                engine.input.borrow_mut().update(&event);
            }
        }
    });
}
