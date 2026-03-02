use winit::event::{Event, WindowEvent};
use egui_winit::State as EguiWinitState;
use egui::{CtxRef, CentralPanel};

pub struct GuiEditor {
    pub egui_state: EguiWinitState,
    pub ctx: CtxRef,
    ai_output: String,
}

impl GuiEditor {
    pub fn new() -> Self {
        let ctx = CtxRef::default();
        let egui_state = EguiWinitState::new(&ctx);
        GuiEditor {
            egui_state,
            ctx,
            ai_output: String::new(),
        }
    }

    pub fn handle_event(&mut self, event: &WindowEvent) -> bool {
        self.egui_state.on_event(event)
    }

    pub fn draw(&self, window: &winit::window::Window) {
        let raw_input = self.egui_state.take_egui_input(window);
        let full_output = self.ctx.run(raw_input, |ctx| {
            CentralPanel::default().show(ctx, |ui| {
                ui.heading("Game Engine MVP");
                if ui.button("Generate Sprite").clicked() {
                    // Trigger UI refresh (actual generation handled by engine)
                }
                ui.separator();
                ui.label(format!("Last AI output: {}", self.ai_output));
            });
        });

        self.egui_state
            .handle_platform_output(window, &self.ctx, full_output.platform_output);
    }

    pub fn set_ai_output(&mut self, output: String) {
        self.ai_output = output;
    }
}
