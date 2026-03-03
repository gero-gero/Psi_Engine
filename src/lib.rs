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
    pub asset_generator: ai::AssetGenerator,
    pub scene: scene::Scene,
}

impl Engine {
    pub fn new(window: Window) -> Self {
        let renderer = render::Renderer::new(&window);
        let input_handler = input::InputHandler::new();
        let physics_world = physics::PhysicsWorld::new();
        let gui_editor = gui::GuiEditor::new(&window);
        let asset_generator = ai::AssetGenerator::new();
        let scene = scene::Scene::new(&renderer.device, &renderer.queue);

        Engine {
            window,
            renderer,
            input_handler,
            physics_world,
            gui_editor,
            asset_generator,
            scene,
        }
    }

    pub async fn update(&mut self) {
        self.input_handler.update();
        self.physics_world.step(1.0 / 60.0);
        let size = self.window.inner_size();
        let window_size = [size.width as f32, size.height as f32];
        self.scene.update(&self.renderer.queue, &self.input_handler, 1.0 / 60.0, window_size);

        // Load workflow list from ComfyUI if requested
        if self.gui_editor.loading_workflows {
            self.gui_editor.loading_workflows = false;
            self.gui_editor.set_ai_output("Loading workflows...".to_string());
            match self.asset_generator.list_workflows().await {
                Ok(workflows) => {
                    let count = workflows.len();
                    self.gui_editor.available_workflows = workflows;
                    self.gui_editor.workflows_loaded = true;
                    self.gui_editor.set_ai_output(format!("Loaded {} workflows", count));
                }
                Err(e) => {
                    eprintln!("Failed to list workflows: {}", e);
                    self.gui_editor.set_ai_output(format!("Error loading workflows: {}", e));
                }
            }
        }

        if self.gui_editor.take_generate_request() {
            let workflow_name = self.gui_editor.workflow_name.clone();
            let prompt_text = self.gui_editor.prompt_text.clone();
            self.gui_editor.set_ai_output("Generating sprite...".to_string());

            match self.asset_generator.generate_sprite(&workflow_name, &prompt_text).await {
                Ok(image_data) => {
                    self.scene.set_sprite_texture(&self.renderer.device, &self.renderer.queue, 0, &image_data);
                    self.gui_editor.set_ai_output(format!("Sprite generated ({} bytes)", image_data.len()));
                }
                Err(e) => {
                    eprintln!("Asset generation error: {}", e);
                    self.gui_editor.set_ai_output(format!("Error: {}", e));
                }
            }
        }
    }

    pub fn render(&mut self) {
        let show_3d = self.gui_editor.show_3d;
        self.renderer.render_frame(&self.scene, &mut self.gui_editor, &self.window, show_3d);
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
