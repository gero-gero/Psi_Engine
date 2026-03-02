use crate::sprite::Sprite;
use crate::model3d::Model3D;

pub struct Scene {
    pub sprites: Vec<Sprite>,
    pub models_3d: Vec<Model3D>,
    pub dragging: bool,
    pub drag_offset_ndc: [f32; 2],
}

impl Scene {
    pub fn new(device: &wgpu::Device) -> Self {
        let sprite = Sprite::new(device);
        let model_3d = Model3D::new_cube(device);
        Scene {
            sprites: vec![sprite],
            models_3d: vec![model_3d],
            dragging: false,
            drag_offset_ndc: [0.0, 0.0],
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, input_handler: &crate::input::InputHandler, _dt: f32, window_size: [f32; 2]) {
        let mouse_ndc = [
            (input_handler.mouse_position[0] as f32 / window_size[0]) * 2.0 - 1.0,
            -((input_handler.mouse_position[1] as f32 / window_size[1]) * 2.0 - 1.0),
        ];

        if input_handler.mouse_left_pressed && !self.dragging {
            self.dragging = true;
            if let Some(sprite) = self.sprites.get(0) {
                self.drag_offset_ndc = [
                    sprite.uniform.position[0] - mouse_ndc[0],
                    sprite.uniform.position[1] - mouse_ndc[1],
                ];
            }
        }

        if !input_handler.mouse_left_pressed {
            self.dragging = false;
        }

        if self.dragging {
            let new_pos = [
                mouse_ndc[0] + self.drag_offset_ndc[0],
                mouse_ndc[1] + self.drag_offset_ndc[1],
            ];
            if let Some(sprite) = self.sprites.get_mut(0) {
                sprite.update_position(queue, new_pos);
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
