use std::usize;

use winit::event::{DeviceEvent, ElementState, Event, MouseScrollDelta, VirtualKeyCode};

use super::engine::Engine;

#[derive(Debug, PartialEq, Copy, Clone)]
struct InputState {
    /// Stores whether a mouse button is held down
    mouse_held: [bool; 16],
    /// Stores whether a key is held down during
    key_held: [bool; 255],
    /// The current mouse position
    mouse_delta: (f64, f64),
    /// Amount of scroll
    scroll_delta: (f32, f32),
}

impl Default for InputState {
    fn default() -> Self {
        InputState {
            mouse_held: [false; 16],
            key_held: [false; 255],
            mouse_delta: (0., 0.),
            scroll_delta: (0., 0.),
        }
    }
}

impl InputState {
    // rolling over to the next frame, deciding which values to keep and which not
    fn rollover(&mut self) {
        self.mouse_delta = (0., 0.);
        self.scroll_delta = (0., 0.);
    }
}

pub struct Input {
    /// The current input state
    state: InputState,
    /// The input state last frame
    state_prev: InputState,
    /// Whether we should keep sending events even when the window is not focused
    events_during_unfocus: bool,
}

impl Input {
    pub(crate) fn new() -> Self {
        Self {
            state: InputState::default(),
            state_prev: InputState::default(),
            events_during_unfocus: false,
        }
    }

    pub(crate) fn update(&mut self, event: &Event<()>, engine: &Engine) {
        // match other events only if the window is focused
        if engine.window.is_focused() || self.events_during_unfocus {
            self.handle_input(event);
        }
    }

    // handle input events
    fn handle_input(&mut self, event: &Event<()>) {
        match event {
            // mouse button
            Event::DeviceEvent {
                event: DeviceEvent::Button { button, state },
                ..
            } => match state {
                ElementState::Pressed => {
                    self.state.mouse_held[*button as usize] = true;
                }
                ElementState::Released => {
                    self.state.mouse_held[*button as usize] = false;
                }
            },
            // mouse motion
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                self.state.mouse_delta = *delta;
            }
            // mouse wheel
            Event::DeviceEvent {
                event: DeviceEvent::MouseWheel { delta },
                ..
            } => match delta {
                MouseScrollDelta::LineDelta(x, y) => {
                    self.state.scroll_delta = (*x, *y);
                }
                // this does not work for some reason
                MouseScrollDelta::PixelDelta(_pos) => {}
            },
            // key press
            Event::DeviceEvent {
                event: DeviceEvent::Key(input),
                ..
            } => {
                if let Some(vcc) = input.virtual_keycode {
                    match input.state {
                        ElementState::Pressed => {
                            self.state.key_held[vcc as usize] = true;
                        }
                        ElementState::Released => {
                            self.state.key_held[vcc as usize] = false;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // handles built-in key presses
    pub(crate) fn handle_builtin(&self, engine: &Engine) {
        if self.get_button_down(VirtualKeyCode::LAlt)
            && self.get_button_was_down(VirtualKeyCode::Return)
        {
            if !engine.window.get_fullscreen() {
                engine.window.set_fullscreen();
            } else {
                engine.window.set_windowed();
            }
        }
    }

    // run this right after the gameloop update
    /// Rolls the input state over to next frame
    pub(crate) fn rollover_state(&mut self) {
        // reset state and assign prev
        self.state_prev = self.state;
        self.state.rollover();
    }

    /// Returns whether the button was pressed this frame
    pub fn get_button_was_down(&self, key: VirtualKeyCode) -> bool {
        !self.state_prev.key_held[key as usize] && self.state.key_held[key as usize]
    }

    /// Returns whether the button is pressed down right now
    pub fn get_button_down(&self, key: VirtualKeyCode) -> bool {
        self.state.key_held[key as usize]
    }

    /// Returns whether the button is not pressed right now
    pub fn get_button_up(&self, key: VirtualKeyCode) -> bool {
        !self.get_button_down(key)
    }

    /// Returns the mouse delta
    pub fn get_mouse_delta(&self) -> (f64, f64) {
        self.state.mouse_delta
    }

    /// Returns the scroll delta
    pub fn get_scroll_delta(&self) -> (f32, f32) {
        self.state.scroll_delta
    }

    /// Set whether events should be recieved even though the window is not focused
    pub fn set_recieve_events_during_unfocus(&mut self, events_during_unfocus: bool) {
        self.events_during_unfocus = events_during_unfocus;
    }
}
