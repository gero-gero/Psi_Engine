use winit::window::Window;
use wgpu::{Device, Queue, Surface};

pub struct Renderer {
    pub surface: Surface,
    pub device: Device,
    pub queue: Queue,
}

impl Renderer {
    pub fn new(window: &Window) -> Self {
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = pollster::block_on(instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ))
        .expect("Failed to find an appropriate adapter");

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None,
        ))
        .expect("Failed to create device");

        Renderer {
            surface,
            device,
            queue,
        }
    }

    pub fn render_frame(&mut self) {
        let output = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(_) => return,
        };

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}
