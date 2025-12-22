use crate::{renderer::Gpu, utils::Shape2};
use eyre::Result;
use std::sync::Arc;
use winit::window::Window;

pub struct Surface {
    device: Arc<Gpu>,
    surface: wgpu::Surface<'static>,
    format: wgpu::TextureFormat,
    size: winit::dpi::PhysicalSize<u32>,
}

impl Surface {
    pub fn new(device: Arc<Gpu>, window: Arc<Window>) -> Result<Self> {
        let size = window.inner_size();

        let surface = device.instance().create_surface(window)?;

        let cap = surface.get_capabilities(device.adapter());
        let format = cap.formats[0];

        let out = Surface { device, surface, format, size };

        out.configure();

        Ok(out)
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    pub fn size_as_shape(&self) -> Shape2 {
        (self.size.height, self.size.width).into()
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.size = size;
        self.configure();
    }

    pub fn texture(&self) -> Result<(wgpu::SurfaceTexture, wgpu::TextureView)> {
        let surface_texture = self.surface.get_current_texture()?;

        let texture_view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(self.format.add_srgb_suffix()),
            ..Default::default()
        });

        Ok((surface_texture, texture_view))
    }

    fn configure(&self) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.format,
            view_formats: vec![self.format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: self.size.width,
            height: self.size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };

        self.surface.configure(&self.device.device(), &surface_config);
    }
}
