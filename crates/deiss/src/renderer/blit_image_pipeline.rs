use crate::{
    renderer::Gpu,
    utils::{RgbaImage, Shape2},
};
use wgpu::{TextureView, util::DeviceExt};

pub struct BlitImagePipeline {
    paint_shape: Shape2,
    paint_tex: wgpu::Texture,
    paint_tex_view: wgpu::TextureView,
    render_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

impl BlitImagePipeline {
    pub fn new(gpu: &Gpu, paint_shape: Shape2) -> Self {
        let vertex_shader_src = include_str!("screen_space_quad.wgsl");
        let fragment_shader_src = include_str!("tonemap.wgsl");
        let source = format!("{vertex_shader_src}\n{fragment_shader_src}");

        let shader = gpu.device().create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Image Blit Shader"),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });

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
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
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

        let render_pipeline_layout =
            gpu.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline =
            gpu.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Image Blit Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Bgra8UnormSrgb,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            });

        let sampler = gpu.device().create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let paint_tex = gpu.device().create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: paint_shape.cols(),
                height: paint_shape.rows(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("Image Texture"),
            view_formats: &[],
        });

        let paint_tex_view = paint_tex.create_view(&wgpu::TextureViewDescriptor::default());

        Self { render_pipeline, bind_group_layout, sampler, paint_shape, paint_tex, paint_tex_view }
    }

    pub fn render(
        &mut self,
        gpu: &Gpu,
        encoder: &mut wgpu::CommandEncoder,
        texture_view: TextureView,
        surface_shape: Shape2,
        image: &RgbaImage,
    ) {
        assert_eq!(image.shape(), self.paint_shape);

        // Upload painted image to texture
        gpu.queue().write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.paint_tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            image.as_bytes(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * image.cols()),
                rows_per_image: Some(image.rows()),
            },
            wgpu::Extent3d {
                width: self.paint_shape.cols(),
                height: self.paint_shape.rows(),
                depth_or_array_layers: 1,
            },
        );

        // Create uniform buffer for aspect ratio calculations
        let uniform_data = [
            image.cols() as f32,         // image_width
            image.rows() as f32,         // image_height
            surface_shape.cols() as f32, // surface_width
            surface_shape.rows() as f32, // surface_height
        ];
        let uniform_buffer = gpu.device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Aspect Ratio Uniform Buffer"),
            contents: bytemuck::cast_slice(&uniform_data),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let bind_group = gpu.device().create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.paint_tex_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry { binding: 2, resource: uniform_buffer.as_entire_binding() },
            ],
            label: Some("texture_bind_group"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
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
