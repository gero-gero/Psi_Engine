use crate::sprite::Sprite;
use crate::model3d::Model3D;

pub struct Scene {
    pub sprites: Vec<Sprite>,
    pub models_3d: Vec<Model3D>,
    pub dragging: bool,
}

impl Scene {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let sprite = Sprite::new(device, queue);
        let model_3d = Model3D::new_cube(device);
        Scene {
            sprites: vec![sprite],
            models_3d: vec![model_3d],
            dragging: false,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, input_handler: &crate::input::InputHandler, _dt: f32, window_size: [f32; 2]) {
        let _mouse_ndc = [
            (input_handler.mouse_position[0] as f32 / window_size[0]) * 2.0 - 1.0,
            -((input_handler.mouse_position[1] as f32 / window_size[1]) * 2.0 - 1.0),
        ];

        if input_handler.mouse_left_pressed && !self.dragging {
            self.dragging = true;
        }

        if !input_handler.mouse_left_pressed {
            self.dragging = false;
        }

        if self.dragging {
            let current_ndc = [
                (input_handler.mouse_position[0] as f32 / window_size[0]) * 2.0 - 1.0,
                -((input_handler.mouse_position[1] as f32 / window_size[1]) * 2.0 - 1.0),
            ];
            let start_ndc = [
                (input_handler.drag_start[0] as f32 / window_size[0]) * 2.0 - 1.0,
                -((input_handler.drag_start[1] as f32 / window_size[1]) * 2.0 - 1.0),
            ];
            let drag_offset = [current_ndc[0] - start_ndc[0], current_ndc[1] - start_ndc[1]];
            self.sprites[0].set_position(queue, drag_offset);
        }
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, depth_view: Option<&wgpu::TextureView>, show_3d: bool) {
        for sprite in &self.sprites {
            sprite.render(encoder, view);
        }
        if show_3d {
            if let Some(depth_view) = depth_view {
                for model in &self.models_3d {
                    model.render(encoder, view, depth_view);
                }
            }
        }
    }

    pub fn set_sprite_texture(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, index: usize, image_data: &[u8]) {
        if let Some(sprite) = self.sprites.get_mut(index) {
            sprite.update_texture(device, queue, image_data);
        }
    }
}
