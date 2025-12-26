use crate::{renderer::Gpu, utils::Shape2};
use egui_wgpu::RendererOptions;

pub struct EguiPipeline {
    state: egui_winit::State,
    scale_factor: f32,
    renderer: egui_wgpu::Renderer,
    frame_started: bool,
}

impl EguiPipeline {
    pub fn context(&self) -> &egui::Context {
        self.state.egui_ctx()
    }

    pub fn new(
        device: &wgpu::Device,
        output_color_format: wgpu::TextureFormat,
        window: &winit::window::Window,
    ) -> EguiPipeline {
        let egui_context = egui::Context::default();

        let egui_state = egui_winit::State::new(
            egui_context,
            egui::viewport::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
            Some(2 * 1024), // default dimension is 2048
        );
        let egui_renderer =
            egui_wgpu::Renderer::new(device, output_color_format, RendererOptions::default());

        EguiPipeline {
            state: egui_state,
            scale_factor: 1.,
            renderer: egui_renderer,
            frame_started: false,
        }
    }

    pub fn handle_input(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::WindowEvent,
    ) -> egui_winit::EventResponse {
        self.state.on_window_event(window, event)
    }

    pub fn ppp(&mut self, v: f32) {
        self.context().set_pixels_per_point(v);
    }

    pub fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    pub fn render(
        &mut self,
        gpu: &Gpu,
        texture_view: &wgpu::TextureView,
        surface_shape: Shape2,
        win: &winit::window::Window,
        f: impl FnOnce(&egui::Context),
    ) {
        let mut encoder = gpu.device().create_command_encoder(&Default::default());

        // render EGUI: render over present
        {
            self.begin_frame(&win);

            f(self.context());

            let screen_desc = egui_wgpu::ScreenDescriptor {
                size_in_pixels: [surface_shape.cols(), surface_shape.rows()],
                pixels_per_point: win.scale_factor() as f32 * self.scale_factor(),
            };

            self.end_frame_and_draw(
                gpu.device(),
                gpu.queue(),
                &mut encoder,
                win,
                texture_view,
                screen_desc,
            );
        }

        gpu.queue().submit([encoder.finish()]);
    }

    pub fn begin_frame(&mut self, window: &winit::window::Window) {
        let raw_input = self.state.take_egui_input(window);
        self.state.egui_ctx().begin_pass(raw_input);
        self.frame_started = true;
    }

    pub fn end_frame_and_draw(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        window: &winit::window::Window,
        window_surface_view: &wgpu::TextureView,
        screen_descriptor: egui_wgpu::ScreenDescriptor,
    ) {
        if !self.frame_started {
            panic!("begin_frame must be called before end_frame_and_draw can be called!");
        }

        self.ppp(screen_descriptor.pixels_per_point);

        let full_output = self.state.egui_ctx().end_pass();

        self.state.handle_platform_output(window, full_output.platform_output);

        let tris = self
            .state
            .egui_ctx()
            .tessellate(full_output.shapes, self.state.egui_ctx().pixels_per_point());
        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer.update_texture(device, queue, *id, image_delta);
        }
        self.renderer.update_buffers(device, queue, encoder, &tris, &screen_descriptor);
        let rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: window_surface_view,
                depth_slice: None,
                resolve_target: None,
                ops: egui_wgpu::wgpu::Operations {
                    load: egui_wgpu::wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            label: Some("egui main render pass"),
            occlusion_query_set: None,
        });

        self.renderer.render(&mut rpass.forget_lifetime(), &tris, &screen_descriptor);
        for x in &full_output.textures_delta.free {
            self.renderer.free_texture(x)
        }

        self.frame_started = false;
    }
}
