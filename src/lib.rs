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
        let scene = scene::Scene::new(&renderer.device);

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

        if self.gui_editor.take_generate_request() {
            match self.asset_generator.generate_sprite("A simple 2D sprite of a red square").await {
                Ok(image_data) => {
                    self.scene.set_sprite_texture(&self.renderer.device, &self.renderer.queue, 0, &image_data);
                    self.gui_editor.set_ai_output("Sprite generated".to_string());
                }
                Err(e) => eprintln!("Asset generation error: {}", e),
            }
        }
    }

    pub fn render(&mut self) {
        self.renderer.render_frame(&self.scene, &mut self.gui_editor, &self.window, self.gui_editor.show_3d);
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
