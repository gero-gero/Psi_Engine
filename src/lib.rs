pub mod render;
pub mod input;
pub mod physics;
pub mod gui;
pub mod ai;

use winit::window::Window;

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
        self.input_handler.update();
        self.physics_world.step(1.0 / 60.0);

        match self.ai_engine.process() {
            Ok(output) => self.gui_editor.set_ai_output(output),
            Err(e) => eprintln!("AI processing error: {}", e),
        }
    }

    pub fn render(&mut self) {
        self.renderer.render_frame();
        self.gui_editor.draw(&self.window);
    }

    pub fn handle_window_event(&mut self, event: &winit::event::WindowEvent) -> bool {
        if self.gui_editor.handle_event(event) {
            return true;
        }
        self.input_handler.handle_event(event)
    }
}
