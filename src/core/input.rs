use std::usize;

use winit::event::{DeviceEvent, ElementState, Event, MouseScrollDelta, VirtualKeyCode};

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
    state: InputState,
    state_prev: InputState,
}

impl Input {
    pub(crate) fn new() -> Self {
        Self {
            state: InputState::default(),
            state_prev: InputState::default(),
        }
    }

    pub(crate) fn update(&mut self, event: &Event<()>) {
        match event {
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
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                self.state.mouse_delta = *delta;
            }
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

    // run this right after the gameloop update
    /// Rolls the input state over to next frame
    pub(crate) fn rollover_state(&mut self) {
        // reset state and assign prev
        // self.state_prev = InputState::default();
        self.state_prev = self.state;
        self.state.rollover();
        // swap(&mut self.state_prev, &mut self.state);
    }

    /// Returns whether the button was pressed this frame
    pub fn button_was_down(&self, key: VirtualKeyCode) -> bool {
        !self.state_prev.key_held[key as usize] && self.state.key_held[key as usize]
    }

    /// Returns whether the button is pressed down right now
    pub fn button_down(&self, key: VirtualKeyCode) -> bool {
        self.state.key_held[key as usize]
    }

    /// Returns whether the button is not pressed right now
    pub fn button_up(&self, key: VirtualKeyCode) -> bool {
        !self.button_down(key)
    }

    /// Returns the mouse delta
    pub fn mouse_delta(&self) -> (f64, f64) {
        self.state.mouse_delta
    }

    /// Returns the scroll delta
    pub fn scroll_delta(&self) -> (f32, f32) {
        self.state.scroll_delta
    }
}
