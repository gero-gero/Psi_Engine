use winit::event::{WindowEvent, VirtualKeyCode, MouseButton, ElementState};

pub struct InputHandler {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub mouse_position: [f64; 2],
    pub mouse_left_pressed: bool,
    pub dragging: bool,
    pub drag_start: [f64; 2],
    pub drag_offset: [f64; 2],
}

impl InputHandler {
    pub fn new() -> Self {
        InputHandler {
            left: false,
            right: false,
            up: false,
            down: false,
            mouse_position: [0.0, 0.0],
            mouse_left_pressed: false,
            dragging: false,
            drag_start: [0.0, 0.0],
            drag_offset: [0.0, 0.0],
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
            WindowEvent::MouseInput { button, state, .. } => {
                if *button == MouseButton::Left {
                    self.mouse_left_pressed = *state == ElementState::Pressed;
                    if *state == ElementState::Pressed {
                        self.dragging = true;
                        self.drag_start = self.mouse_position;
                        self.drag_offset = [0.0, 0.0];
                    } else {
                        self.dragging = false;
                    }
                    true
                } else {
                    false
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position = [position.x, position.y];
                true
            }
            _ => false,
        }
    }

    pub fn get_velocity(&self) -> [f32; 2] {
        let mut vx = 0.0;
        let mut vy = 0.0;
        if self.left {
            vx -= 1.0;
        }
        if self.right {
            vx += 1.0;
        }
        if self.up {
            vy += 1.0;
        }
        if self.down {
            vy -= 1.0;
        }
        [vx, vy]
    }

    pub fn get_drag_position(&self, window_size: [f32; 2]) -> [f32; 2] {
        if self.dragging {
            let x = (self.mouse_position[0] as f32 / window_size[0]) * 2.0 - 1.0;
            let y = -((self.mouse_position[1] as f32 / window_size[1]) * 2.0 - 1.0);
            [x, y]
        } else {
            [0.0, 0.0]
        }
    }
}
