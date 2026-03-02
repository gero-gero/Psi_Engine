pub mod render;
pub mod input;
pub mod physics;
pub mod gui;
pub mod ai;
pub mod sprite;
pub mod model3d;
pub mod scene;

use winit::window::Window;

pub struct Engine {
    pub window: Window,
    pub renderer: render::Renderer,
    pub input_handler: input::InputHandler,
    pub physics_world: physics::PhysicsWorld,
    pub gui_editor: gui::GuiEditor,
    pub ai_engine: ai::LLMEngine,
    pub scene: scene::Scene,
}

impl Engine {
    pub fn new(window: Window) -> Self {
        let renderer = render::Renderer::new(&window);
        let input_handler = input::InputHandler::new();
        let physics_world = physics::PhysicsWorld::new();
        let gui_editor = gui::GuiEditor::new(&window);
        let ai_engine = ai::LLMEngine::new();
        let scene = scene::Scene::new(&renderer.device);

        Engine {
            window,
            renderer,
            input_handler,
            physics_world,
            gui_editor,
            ai_engine,
            scene,
        }
    }

    pub async fn update(&mut self) {
        self.input_handler.update();
        self.physics_world.step(1.0 / 60.0);
        self.scene.update(&self.renderer.queue, &self.input_handler, 1.0 / 60.0);

        if self.gui_editor.take_generate_request() {
            match self.ai_engine.process().await {
                Ok(output) => {
                    self.gui_editor.set_ai_output(output.clone());
                    let color = ai::LLMEngine::parse_color(&output);
                    self.scene.set_sprite_color(&self.renderer.queue, 0, color);
                }
                Err(e) => eprintln!("AI processing error: {}", e),
            }
        }
    }

    pub fn render(&mut self) {
        self.renderer.render_frame(&self.scene, self.gui_editor.show_3d);
        self.gui_editor.draw(&self.window);
    }

    pub fn handle_window_event(&mut self, event: &winit::event::WindowEvent) -> bool {
        if self.gui_editor.handle_event(event) {
            return true;
        }
        self.input_handler.handle_event(event)
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.renderer.resize(size);
    }
}
