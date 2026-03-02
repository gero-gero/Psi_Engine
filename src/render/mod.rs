use winit::window::Window;
use wgpu::{Device, Queue, Surface};

pub struct Renderer {
    pub surface: Surface,
    pub device: Device,
    pub queue: Queue,
}

impl Renderer {
    pub fn new(window: &Window) -> Self {
        // Initialize GPU
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter_future = async {
            instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
        };

        // For simplicity, block on the future
        let adapter = pollster::block_on(adapter_future).expect("Failed to find an appropriate adapter");

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None,
        ))
        .expect("Failed to create device");

        Renderer { surface, device, queue }
    }

    pub fn render_frame(&mut self) {
        // Acquire frame
        let output = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(_) => return,
        };

        // Create a dummy encoder and submit (no actual rendering)
        let mut encoder =
            self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // Submit commands
        self.queue.submit(std::iter::once(encoder.finish()));

        // Present frame
        output.present();
    }
}
