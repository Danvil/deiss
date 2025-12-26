use crate::{
    painter::Settings,
    renderer::{CrtPipeline, EguiPipeline, Gpu, PresentPipeline},
    utils::{RgbaImage, Shape2},
};
use winit::{event::WindowEvent, window::Window};

pub struct Renderer {
    crt_pipeline: CrtPipeline,
    present_pipeline: PresentPipeline,
    egui_pipeline: EguiPipeline,
}

impl Renderer {
    pub fn new(gpu: &Gpu, window: &Window, paint_shape: Shape2, display_shape: Shape2) -> Self {
        let crt_pipeline = CrtPipeline::new(gpu, paint_shape, display_shape);

        let present_pipeline = PresentPipeline::new(gpu);

        let egui_pipeline =
            EguiPipeline::new(gpu.device(), wgpu::TextureFormat::Bgra8UnormSrgb, window);

        Self { crt_pipeline, present_pipeline, egui_pipeline }
    }

    pub fn handle_input(
        &mut self,
        window: &Window,
        event: &WindowEvent,
    ) -> egui_winit::EventResponse {
        self.egui_pipeline.handle_input(window, event)
    }

    pub fn render_img(
        &mut self,
        gpu: &Gpu,
        texture_view: &wgpu::TextureView,
        display_shape: Shape2,
        image: &RgbaImage,
        settings: &Settings,
    ) {
        let mut encoder = gpu.device().create_command_encoder(&Default::default());

        self.crt_pipeline.render(
            gpu,
            &mut encoder,
            image,
            display_shape,
            &settings.crt_shader_settings,
        );
        self.present_pipeline.render(gpu, &mut encoder, self.crt_pipeline.output(), texture_view);

        gpu.queue().submit([encoder.finish()]);
    }

    pub fn render_gui(
        &mut self,
        gpu: &Gpu,
        texture_view: &wgpu::TextureView,
        surface_shape: Shape2,
        win: &Window,
        f: impl FnOnce(&egui::Context),
    ) {
        self.egui_pipeline.render(gpu, texture_view, surface_shape, win, f)
    }
}
