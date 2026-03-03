use winit::window::Window;
use wgpu::{Device, Queue, Surface, Adapter};
use egui_wgpu::Renderer as EguiRenderer;

pub struct Renderer {
    pub surface: Surface,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
    pub depth_texture: wgpu::Texture,
    pub depth_view: wgpu::TextureView,
    pub config: wgpu::SurfaceConfiguration,
    pub egui_renderer: EguiRenderer,
}

impl Renderer {
    pub fn new(window: &Window) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface = unsafe { instance.create_surface(window) }.expect("Failed to create surface");
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

        let size = window.inner_size();
        let format = wgpu::TextureFormat::Bgra8UnormSrgb;
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let egui_renderer = EguiRenderer::new(&device, format, None, 1);

        Renderer {
            surface,
            adapter,
            device,
            queue,
            depth_texture,
            depth_view,
            config,
            egui_renderer,
        }
    }

    pub fn render_frame(&mut self, scene: &crate::scene::Scene, gui_editor: &mut crate::gui::GuiEditor, window: &winit::window::Window, show_3d: bool) {
        let output = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(_) => return,
        };
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        scene.render(&mut encoder, &view, Some(&self.depth_view), show_3d);

        let raw_input = gui_editor.egui_state.take_egui_input(window);
        let full_output = gui_editor.ctx.run(raw_input, |ctx| {
            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                ui.painter().rect_filled(ui.available_rect_before_wrap(), 0.0, egui::Color32::BLUE);
                ui.heading("Game Engine MVP");
                if ui.button("Generate Sprite").clicked() {
                    gui_editor.generate_requested = true;
                }
                ui.checkbox(&mut gui_editor.show_3d, "Show 3D Cube");
                ui.separator();
                ui.label(format!("Last output: {}", gui_editor.ai_output));
                ui.label("Left click and drag to move the sprite.");
            });

            egui::SidePanel::right("right_panel").show(ctx, |ui| {
                ui.painter().rect_filled(ui.available_rect_before_wrap(), 0.0, egui::Color32::GREEN);
                ui.label("Text Box:");
                ui.text_edit_singleline(&mut gui_editor.text_box);
            });
        });

        gui_editor.egui_state
            .handle_platform_output(window, &gui_editor.ctx, full_output.platform_output);

        let paint_jobs = gui_editor.ctx.tessellate(full_output.shapes);

        let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: window.scale_factor() as f32,
        };

        self.egui_renderer.update_buffers(&self.device, &self.queue, &mut encoder, &paint_jobs, &screen_descriptor);

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Egui Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            self.egui_renderer.render(&mut render_pass, &paint_jobs, &screen_descriptor);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
        // Recreate depth texture
        self.depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        self.depth_view = self.depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
    }
}
