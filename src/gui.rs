
use egui::Context;
use wgpu::Device;
use winit::event::WindowEvent;
use winit::window::Window;
use egui_wgpu::{ScreenDescriptor, Renderer};

pub struct EguiAdapter {
    renderer: Renderer,
    state: egui_winit::State,
    context: egui::Context,
}

impl EguiAdapter {

    pub fn new(
        device: &Device,
        surface_config: &wgpu::SurfaceConfiguration,
        window: &Window,
    ) -> Self {
        let egui_context = Context::default();
        let vid = egui_context.viewport_id();

        const BORDER_RADIUS: f32 = 2.0;

        let visual = egui::Visuals {
            window_rounding: egui::Rounding::same(BORDER_RADIUS),
            window_shadow: egui::epaint::Shadow::NONE,
            // menu_rounding: todo!(),
            ..Default::default()
        };

        egui_context.set_visuals(visual);
        let native_pixels_per_point = window.scale_factor() as f32;
        let max_texture_side = device.limits().max_texture_dimension_2d as usize;
        let egui_state = egui_winit::State::new(egui_context.clone(), vid, window, Some(native_pixels_per_point), Some(max_texture_side));

        let renderer = Renderer::new(
            device,
            surface_config.format,
            None,
            1);
        Self {
            context: egui_context,
            state: egui_state,
            renderer,
        }
    }

    pub fn handle_input(&mut self, window: &Window, event: &WindowEvent) {
        let _ = self.state.on_window_event(window, event);
    }

    pub fn draw(
        &mut self,
        device: &Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        window: &Window,
        window_surface_view: &wgpu::TextureView,
        screen_descriptor: ScreenDescriptor,
        run_ui: impl FnOnce(&egui::Context),
    ) {
        // self.state.set_pixels_per_point(window.scale_factor() as f32);
        let raw_input = self.state.take_egui_input(&window);
        let full_output = self.context.run(raw_input, run_ui);

        self.state
            .handle_platform_output(&window, full_output.platform_output);

        let tris = self
            .context
            .tessellate(full_output.shapes, full_output.pixels_per_point);
        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(&device, &queue, *id, &image_delta);
        }
        self.renderer
            .update_buffers(&device, &queue, encoder, &tris, &screen_descriptor);
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &window_surface_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            label: Some("egui main render pass"),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        self.renderer.render(&mut rpass, &tris, &screen_descriptor);
        drop(rpass);
        for x in &full_output.textures_delta.free {
            self.renderer.free_texture(x)
        }
    }

}