use crate::sprite::Sprite;
use crate::model3d::Model3D;
use wgpu::Queue;

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

    pub fn update(&mut self, queue: &Queue, input_handler: &crate::input::InputHandler, dt: f32) {
        for sprite in &mut self.sprites {
            let mut position = sprite.uniform.position;
            if input_handler.left {
                position[0] -= 1.0 * dt;
            }
            if input_handler.right {
                position[0] += 1.0 * dt;
            }
            if input_handler.up {
                position[1] += 1.0 * dt;
            }
            if input_handler.down {
                position[1] -= 1.0 * dt;
            }
            sprite.update_position(queue, position);
        }
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, depth_view: Option<&wgpu::TextureView>) {
        for sprite in &self.sprites {
            sprite.render(encoder, view);
        }
        if let Some(depth_view) = depth_view {
            for model in &self.models_3d {
                model.render(encoder, view, depth_view);
            }
        }
    }

    pub fn set_sprite_color(&mut self, queue: &Queue, index: usize, color: [f32; 4]) {
        if let Some(sprite) = self.sprites.get_mut(index) {
            sprite.set_color(queue, color);
        }
    }
}
