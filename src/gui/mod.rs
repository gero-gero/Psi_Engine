use winit::event::WindowEvent;
use egui_winit::State as EguiWinitState;
use egui::{Context, TopBottomPanel};

pub struct GuiEditor {
    pub egui_state: EguiWinitState,
    pub ctx: Context,
    ai_output: String,
    pub generate_requested: bool,
    pub show_3d: bool,
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
        }
    }

    pub fn handle_event(&mut self, event: &WindowEvent) -> bool {
        let response = self.egui_state.on_event(&self.ctx, event);
        response.consumed
    }

    pub fn draw(&mut self, window: &winit::window::Window) {
        let raw_input = self.egui_state.take_egui_input(window);
        let full_output = self.ctx.run(raw_input, |ctx| {
            TopBottomPanel::top("top_panel").show(ctx, |ui| {
                ui.heading("Game Engine MVP");
                if ui.button("Generate Sprite").clicked() {
                    self.generate_requested = true;
                }
                ui.checkbox(&mut self.show_3d, "Show 3D Cube");
                ui.separator();
                ui.label(format!("Last output: {}", self.ai_output));
                ui.label("Left click and drag to move the sprite.");
            });
        });

        self.egui_state
            .handle_platform_output(window, &self.ctx, full_output.platform_output);
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
