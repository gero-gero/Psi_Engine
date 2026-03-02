use winit::event::{Event, WindowEvent};

pub struct InputHandler;

impl InputHandler {
    pub fn new() -> Self {
        InputHandler
    }

    pub fn update(&mut self) {
        // Process input state (placeholder)
    }

    pub fn handle_event(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { .. } => {
                // Handle keyboard
                true
            }
            _ => false,
        }
    }
}
