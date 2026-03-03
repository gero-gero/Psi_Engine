use winit::event::WindowEvent;
use egui_winit::State as EguiWinitState;
use egui::{Context};

pub struct GuiEditor {
    pub egui_state: EguiWinitState,
    pub ctx: Context,
    pub ai_output: String,
    pub generate_requested: bool,
    pub show_3d: bool,
    pub text_box: String,
}

impl GuiEditor {
    pub fn new(window: &winit::window::Window) -> Self {
        let ctx = Context::default();
        let egui_state = EguiWinitState::new(window);
        GuiEditor {
            egui_state,
            ctx,
            ai_output: String::new(),
            generate_requested: false,
            show_3d: false,
            text_box: String::new(),
        }
    }

    pub fn handle_event(&mut self, event: &WindowEvent) -> bool {
        let response = self.egui_state.on_event(&self.ctx, event);
        response.consumed
    }

    pub fn set_ai_output(&mut self, output: String) {
        self.ai_output = output;
    }

    pub fn take_generate_request(&mut self) -> bool {
        let req = self.generate_requested;
        self.generate_requested = false;
        req
    }
}
