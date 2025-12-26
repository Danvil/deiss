use crate::{
    renderer::{BlitImagePipeline, EguiPipeline, Gpu},
    utils::{RgbaImage, Shape2},
};
use winit::{event::WindowEvent, window::Window};

pub struct Renderer {
    blit_image_pipeline: BlitImagePipeline,
    egui_pipeline: EguiPipeline,
}

impl Renderer {
    pub fn new(gpu: &Gpu, window: &Window, paint_shape: Shape2, display_shape: Shape2) -> Self {
        let blit_image_pipeline = BlitImagePipeline::new(gpu, paint_shape);

        let egui_pipeline =
            EguiPipeline::new(gpu.device(), wgpu::TextureFormat::Bgra8UnormSrgb, window);

        Self { blit_image_pipeline, egui_pipeline }
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
        texture_view: wgpu::TextureView,
        surface_shape: Shape2,
        image: &RgbaImage,
    ) {
        let mut encoder = gpu.device().create_command_encoder(&Default::default());

        self.blit_image_pipeline.render(gpu, &mut encoder, texture_view, surface_shape, image);

        gpu.queue().submit([encoder.finish()]);
    }

    pub fn render_gui(
        &mut self,
        gpu: &Gpu,
        texture_view: wgpu::TextureView,
        surface_shape: Shape2,
        win: &Window,
        f: impl FnOnce(&egui::Context),
    ) {
        self.egui_pipeline.render(gpu, texture_view, surface_shape, win, f)
    }
}
