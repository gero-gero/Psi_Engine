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
    pub gui_renderer: GuiRenderer,
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

        let gui_renderer = GuiRenderer::new(&device, format);

        Renderer {
            surface,
            adapter,
            device,
            queue,
            depth_texture,
            depth_view,
            config,
            gui_renderer,
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

        self.gui_renderer.render(&mut encoder, &view, gui_editor, window, &self.device, &self.queue);

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

pub struct GuiRenderer {
    egui_renderer: EguiRenderer,
}

impl GuiRenderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let egui_renderer = EguiRenderer::new(device, format, None, 1);
        GuiRenderer { egui_renderer }
    }

    pub fn render(&mut self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, gui_editor: &mut crate::gui::GuiEditor, window: &winit::window::Window, device: &wgpu::Device, queue: &wgpu::Queue) {
        let raw_input = gui_editor.egui_state.take_egui_input(window);
        let full_output = gui_editor.ctx.run(raw_input, |ctx| {
            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                ui.heading("Game Engine MVP");

                ui.horizontal(|ui| {
                    ui.checkbox(&mut gui_editor.show_3d, "Show 3D Cube");
                    ui.separator();
                    ui.label(format!("Status: {}", gui_editor.ai_output));
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Workflow:");
                    ui.add(egui::TextEdit::singleline(&mut gui_editor.workflow_name).desired_width(200.0));

                    if !gui_editor.available_workflows.is_empty() {
                        egui::ComboBox::from_id_source("workflow_combo")
                            .selected_text(if gui_editor.workflow_name.is_empty() {
                                "Select...".to_string()
                            } else {
                                gui_editor.workflow_name.clone()
                            })
                            .show_ui(ui, |ui| {
                                let workflows = gui_editor.available_workflows.clone();
                                for wf in workflows {
                                    ui.selectable_value(&mut gui_editor.workflow_name, wf.clone(), &wf);
                                }
                            });
                    }

                    if ui.button("Refresh Workflows").clicked() {
                        gui_editor.loading_workflows = true;
                    }
                    ui.small("(from workflows/ folder)");
                });

                ui.horizontal(|ui| {
                    ui.label("Prompt:");
                    ui.add(egui::TextEdit::singleline(&mut gui_editor.prompt_text).desired_width(400.0));

                    if ui.button("Generate Sprite").clicked() {
                        if !gui_editor.workflow_name.is_empty() && !gui_editor.prompt_text.is_empty() {
                            gui_editor.generate_requested = true;
                        }
                    }
                });

                ui.horizontal(|ui| {
                    ui.checkbox(&mut gui_editor.auto_crop, "Auto-crop background");
                    if gui_editor.auto_crop {
                        ui.label("Tolerance:");
                        let mut tol = gui_editor.crop_tolerance as f32;
                        ui.add(egui::Slider::new(&mut tol, 5.0..=100.0).integer());
                        gui_editor.crop_tolerance = tol as u8;
                    }
                });

                ui.label("Left click and drag to move sprites.");
            });
        });

        gui_editor.egui_state
            .handle_platform_output(window, &gui_editor.ctx, full_output.platform_output);

        static mut CHECKED: bool = false;
        unsafe {
            if !CHECKED {
                if full_output.shapes.is_empty() {
                    eprintln!("Error: GUI not producing shapes - GUI rendering failed");
                } else {
                    println!("GUI is producing {} shapes", full_output.shapes.len());
                }
                CHECKED = true;
            }
        }

        let paint_jobs = gui_editor.ctx.tessellate(full_output.shapes);

        let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
            size_in_pixels: [window.inner_size().width, window.inner_size().height],
            pixels_per_point: window.scale_factor() as f32,
        };

        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_renderer.update_texture(device, queue, *id, image_delta);
        }

        self.egui_renderer.update_buffers(device, queue, encoder, &paint_jobs, &screen_descriptor);

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Egui Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
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

        for id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }
    }
}
