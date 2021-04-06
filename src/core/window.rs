use crate::core::engine::EngineInit;
use winit::event::Event;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct InitialWindowInfo {
    pub initial_dimensions: Dimensions,
    pub title: &'static str,
}

impl InitialWindowInfo {
    pub(crate) fn build<T: 'static>(
        self,
        window_target: &winit::event_loop::EventLoopWindowTarget<T>,
    ) -> Result<Window, winit::error::OsError> {
        let winit_window = winit::window::WindowBuilder::new()
            .with_title(self.title)
            .with_inner_size(winit::dpi::LogicalSize::new(
                f64::from(self.initial_dimensions.width),
                f64::from(self.initial_dimensions.height),
            ))
            .with_min_inner_size(winit::dpi::LogicalSize::new(64, 64))
            .build(window_target)?;
        Ok(Window::new(winit_window))
    }
}

pub struct Window {
    /// The winit window
    pub(crate) winit_window: winit::window::Window,
    /// Whether the cursor is visible and captured
    capture_cursor: bool,
    /// Whether the window is currently in focus
    focused: bool,
}

impl Window {
    fn new(winit_window: winit::window::Window) -> Self {
        Self {
            winit_window,
            capture_cursor: true,
            focused: true, // in the beginning, the windows is always focused
        }
    }

    /// Returns whether the window is currently in focus
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Set the visibility of the mouse cursor and wether it should be captured (when window is focused)
    pub fn set_capture_cursor(&mut self, capture: bool) {
        self.capture_cursor = capture;
        self.actually_capture_cursor(capture);
    }

    // actually perform the capture
    fn actually_capture_cursor(&mut self, capture: bool) {
        self.winit_window.set_cursor_visible(!capture);
        self.winit_window.set_cursor_grab(capture).unwrap();
    }

    // window is started with focused state
    fn on_start(&mut self) {
        self.actually_capture_cursor(self.capture_cursor);
    }

    // conditionally set the focus visibility depending on the user's choice on it's visibility
    // we want the cursor to be visible, whenever the window is not active
    fn on_focus(&mut self, focus: bool) {
        if focus {
            // gained focus
            if self.capture_cursor {
                self.actually_capture_cursor(true);
            }
        } else {
            // lost focus
            self.actually_capture_cursor(false);
        }
    }
}

pub fn start(engine_init: EngineInit) -> ! {
    let mut engine = engine_init.engine;
    engine.window.on_start();
    engine_init.eventloop.run(move |event, _, controlflow| {
        *controlflow = winit::event_loop::ControlFlow::Poll;
        engine.input.borrow_mut().update(&event, &engine);
        match event {
            // close
            Event::WindowEvent {
                event: winit::event::WindowEvent::CloseRequested,
                ..
            } => *controlflow = winit::event_loop::ControlFlow::Exit,
            // focus
            Event::WindowEvent {
                event: winit::event::WindowEvent::Focused(state),
                ..
            } => {
                engine.window.on_focus(state);
            }
            // render
            Event::MainEventsCleared => {
                engine
                    .gameloop
                    .update(&mut engine.vulkan_manager, &engine.scene);
                engine.render();
                engine.input.borrow_mut().rollover_state();
            }
            _ => {}
        }
    });
}
