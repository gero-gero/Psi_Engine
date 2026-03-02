use crate::sprite::Sprite;
use crate::model3d::Model3D;

pub struct Scene {
    pub sprites: Vec<Sprite>,
    pub models_3d: Vec<Model3D>,
}

impl Scene {
    pub fn new(device: &wgpu::Device) -> Self {
        let sprite = Sprite::new(device);
        let model_3d = Model3D::new_cube(device);
        Scene {
            sprites: vec![sprite],
            models_3d: vec![model_3d],
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, input_handler: &crate::input::InputHandler, _dt: f32, window_size: [f32; 2]) {
        if input_handler.dragging {
            let drag_pos = input_handler.get_drag_position(window_size);
            if let Some(sprite) = self.sprites.get_mut(0) {
                sprite.update_position(queue, drag_pos);
            }
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

    pub fn set_sprite_color(&mut self, _queue: &wgpu::Queue, _index: usize, _color: [f32; 4]) {
        // No color change for now
    }
}
