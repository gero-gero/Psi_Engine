pub mod render;
pub mod input;
pub mod physics;
pub mod gui;
pub mod ai;

use winit::{
    window::{Window, WindowBuilder},
};

pub struct Engine {
    pub window: Window,
    pub renderer: render::Renderer,
    pub input_handler: input::InputHandler,
    pub physics_world: physics::PhysicsWorld,
    pub gui_editor: gui::GuiEditor,
    pub ai_engine: ai::LLMEngine,
}

impl Engine {
    pub fn new(window: Window) -> Self {
        let renderer = render::Renderer::new(&window);
        let input_handler = input::InputHandler::new();
        let physics_world = physics::PhysicsWorld::new();
        let gui_editor = gui::GuiEditor::new();
        let ai_engine = ai::LLMEngine::new();

        Engine {
            window,
            renderer,
            input_handler,
            physics_world,
            gui_editor,
            ai_engine,
        }
    }

    pub fn update(&mut self) {
        // Update input
        self.input_handler.update();

        // Update physics
        self.physics_world.step(1.0 / 60.0);

        // Update AI (placeholder)
        self.ai_engine.process();
    }

    pub fn render(&mut self) {
        self.renderer.render_frame();
        self.gui_editor.draw(&self.window);
    }

    pub fn handle_window_event(&mut self, event: &winit::event::WindowEvent) -> bool {
        // Let GUI handle events first
        if self.gui_editor.handle_event(event) {
            return true;
        }
        // Pass to input handler
        self.input_handler.handle_event(event)
    }
}
