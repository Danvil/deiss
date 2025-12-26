use crate::{
    renderer::{Gpu, Shader, Texture, create_pipeline},
    utils::{RgbaImage, Shape2, Vec2},
};
use wgpu::util::DeviceExt;

#[derive(Debug, Clone)]
pub struct CrtShaderSettings {
    pub warp_enabled: bool,
    pub warp_strength: f32,
    pub warp_xy: Vec2,
    pub scanlines_enabled: bool,
    pub scanline_strength: f32,
    pub afterglow_enabled: bool,
    pub afterglow: f32,
}

impl Default for CrtShaderSettings {
    fn default() -> Self {
        Self {
            warp_enabled: true,
            warp_strength: 0.75,
            warp_xy: Vec2::new(0.3, 0.4),
            scanlines_enabled: true,
            scanline_strength: 0.75,
            afterglow_enabled: true,
            afterglow: 0.07,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CrtShaderUniform {
    pub paint_shape: [f32; 2],
    pub display_shape: [f32; 2],
    pub warp: [f32; 2],
    pub scan: f32,
    pub afterglow: f32,
    pub dummy: [f32; 0],
}

impl CrtShaderUniform {
    fn from_shape_settings(
        paint_shape: Shape2,
        display_shape: Shape2,
        settings: &CrtShaderSettings,
    ) -> Self {
        Self {
            paint_shape: [paint_shape.rows() as f32, paint_shape.cols() as f32],
            display_shape: [display_shape.rows() as f32, display_shape.cols() as f32],
            warp: if settings.warp_enabled {
                [
                    settings.warp_strength * settings.warp_xy.x,
                    settings.warp_strength * settings.warp_xy.y,
                ]
            } else {
                [0.; 2]
            },
            scan: if settings.scanlines_enabled { settings.scanline_strength } else { 0. },
            afterglow: if settings.afterglow_enabled { settings.afterglow } else { 0.0 },
            dummy: [0.; 0],
        }
    }
}

/// Retro CRT effect
pub struct CrtPipeline {
    paint_tex: Texture,
    prev_tex: Texture,
    display_tex: Texture,
    render_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

impl CrtPipeline {
    pub fn output(&self) -> &Texture {
        &self.display_tex
    }

    pub fn new(gpu: &Gpu, paint_shape: Shape2, display_shape: Shape2) -> Self {
        let bind_group_layout =
            gpu.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let render_pipeline = create_pipeline(
            gpu,
            "crt",
            &Shader {
                vertex_shader_src: include_str!("screen_space_quad.wgsl"),
                vertex_entry_point: "vs_main",
                fragment_shader_src: include_str!("crt.wgsl"),
                fragment_entry_point: "fs_main",
            },
            &bind_group_layout,
            wgpu::TextureFormat::Rgba16Float,
        );

        let sampler = gpu.device().create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let paint_tex = Texture::new(
            gpu,
            paint_shape,
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );

        let prev_tex = Texture::new(
            gpu,
            display_shape,
            wgpu::TextureFormat::Rgba16Float,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );

        let display_tex = Texture::new(
            gpu,
            display_shape,
            wgpu::TextureFormat::Rgba16Float,
            wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
        );

        Self { paint_tex, prev_tex, display_tex, render_pipeline, bind_group_layout, sampler }
    }

    pub fn render(
        &mut self,
        gpu: &Gpu,
        encoder: &mut wgpu::CommandEncoder,
        image: &RgbaImage,
        display_shape: Shape2,

        settings: &CrtShaderSettings,
    ) {
        self.paint_tex.write(gpu, image);

        self.prev_tex.reshape(gpu, display_shape);
        self.display_tex.reshape(gpu, display_shape);

        self.prev_tex.copy_from(encoder, &self.display_tex);

        // Create uniform buffer for settings
        let uniform_buffer = gpu.device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("CRT Shader Settings Uniform Buffer"),
            contents: bytemuck::cast_slice(&[CrtShaderUniform::from_shape_settings(
                image.shape(),
                display_shape,
                settings,
            )]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let bind_group = gpu.device().create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.paint_tex),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&self.prev_tex),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry { binding: 4, resource: uniform_buffer.as_entire_binding() },
            ],
            label: Some("texture_bind_group"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.display_tex,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }
    }
}
