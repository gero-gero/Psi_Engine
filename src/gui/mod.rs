use winit::event::{Event, WindowEvent};
use egui_winit::State as EguiWinitState;

pub struct GuiEditor {
    pub egui_state: EguiWinitState,
}

impl GuiEditor {
    pub fn new() -> Self {
        let egui_state = EguiWinitState::new();
        GuiEditor { egui_state }
    }

    pub fn handle_event(&mut self, event: &WindowEvent) -> bool {
        // Let egui consume the event
        false
    }

    pub fn draw(&self, window: &winit::window::Window) {
        // Placeholder for drawing GUI
    }
}
