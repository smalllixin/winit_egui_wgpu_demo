use egui_demo_lib::DemoWindows;
use egui_wgpu::ScreenDescriptor;
use log::info;
use wgpu::{Device, Queue, Surface};
use winit::event::WindowEvent;
use winit::window::Window;
use crate::gui::EguiAdapter;

pub struct GpuApp<'a> {
    surface: Surface<'a>,
    device: Device,
    queue: Queue,
    config: wgpu::SurfaceConfiguration,
    graphics_backend: String,

    pub size: winit::dpi::PhysicalSize<u32>,
    window: &'a Window,

    egui_adapter: EguiAdapter,

    demo_windows: DemoWindows,
}


impl <'a> GpuApp<'a> {

    pub async fn new(window: &'a Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let info = adapter.get_info();
        info!(target: "lx", "selected graphics device: {:?}", &info);
        let graphics_backend = format!("{:?}", &info.backend);

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: adapter.features(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                },
                None, // Trace path
            )
            .await
            .unwrap();

        // Set error handler for wgpu errors
        // This is better for use than their default because it includes the error in
        // the panic message
        // device.on_uncaptured_error(Box::new(move |error| {
        //     error!("{}", &error);
        //     panic!(
        //         "wgpu error (handling all wgpu errors as fatal):\n{:?}\n{:?}",
        //         &error, &info,
        //     );
        // }));

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an Srgb surface texture. Using a different
        // one will result all the colors comming out darker. If you want to support non
        // Srgb surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let egui_adapter = EguiAdapter::new(&device, &config, window);

        Self {
            surface,
            device,
            queue,
            config,
            graphics_backend,
            size,
            window,
            egui_adapter,
            demo_windows: DemoWindows::default(),
        }
    }

    pub fn window(&self) -> &'a Window {
        self.window
    }

    pub fn window_mut(&mut self) -> &'a Window {
        self.window
    }

    pub fn input(&mut self, _event: &WindowEvent) -> bool {
        self.window().request_redraw();
        false
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }
        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn update(&mut self) {

    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 1.0,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                }),],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Here's app specific rendering
        }

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: self.window().scale_factor() as f32,
        };
        self.egui_adapter.draw(
            &self.device,
            &self.queue,
            &mut encoder,
            &self.window,
            &view,
            screen_descriptor,
            |ctx| {
                self.demo_windows.ui(ctx);
                // egui::Window::new("Streamline CFD")
                //     // .vscroll(true)
                //     .default_open(true)
                //     .max_width(1000.0)
                //     .max_height(800.0)
                //     .default_width(800.0)
                //     .resizable(true)
                //     .anchor(egui::Align2::LEFT_TOP, [0.0, 0.0])
                //     .show(&ctx, |mut ui| {
                //         if ui.add(egui::Button::new("Click me")).clicked() {
                //             println!("PRESSED")
                //         }
                //
                //         ui.label("Slider");
                //         // ui.add(egui::Slider::new(_, 0..=120).text("age"));
                //         ui.end_row();
                //
                //         // proto_scene.egui(ui);
                //     });
            },
        );

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    pub fn egui_adapter(&mut self) -> &mut EguiAdapter {
        &mut self.egui_adapter
    }

    #[allow(dead_code)]
    pub fn graphics_backend(&self) -> &str {
        &self.graphics_backend
    }
}