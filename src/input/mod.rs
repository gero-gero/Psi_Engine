use winit::event::{WindowEvent, VirtualKeyCode};

pub struct InputHandler {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
}

impl InputHandler {
    pub fn new() -> Self {
        InputHandler {
            left: false,
            right: false,
            up: false,
            down: false,
        }
    }

    pub fn update(&mut self) {
        // Process input state
    }

    pub fn handle_event(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { input, .. } => {
                if let Some(keycode) = input.virtual_keycode {
                    match keycode {
                        VirtualKeyCode::A => self.left = input.state == winit::event::ElementState::Pressed,
                        VirtualKeyCode::D => self.right = input.state == winit::event::ElementState::Pressed,
                        VirtualKeyCode::W => self.up = input.state == winit::event::ElementState::Pressed,
                        VirtualKeyCode::S => self.down = input.state == winit::event::ElementState::Pressed,
                        _ => {}
                    }
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}
