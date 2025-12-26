use std::ops;

use wgpu::Origin3d;

use crate::{
    renderer::Gpu,
    utils::{RgbaImage, Shape2},
};

pub struct Shader<'a> {
    pub vertex_shader_src: &'a str,
    pub vertex_entry_point: &'a str,
    pub fragment_shader_src: &'a str,
    pub fragment_entry_point: &'a str,
}

pub fn create_pipeline(
    gpu: &Gpu,
    label: &str,
    shader: &Shader<'_>,
    bind_group_layout: &wgpu::BindGroupLayout,
    format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let source = format!("{}\n{}", shader.vertex_shader_src, shader.fragment_shader_src);

    let shader_module = gpu.device().create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(&format!("{label} shader")),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });

    let render_pipeline_layout =
        gpu.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{label} render pipeline layout")),
            bind_group_layouts: &[bind_group_layout],
            push_constant_ranges: &[],
        });

    let render_pipeline = gpu.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(&format!("{label} render pipeline")),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: Some(shader.vertex_entry_point),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: Some(shader.fragment_entry_point),
            targets: &[Some(wgpu::ColorTargetState {
                format,
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

    render_pipeline
}

pub struct Texture {
    shape: Shape2,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
    tex: wgpu::Texture,
    view: wgpu::TextureView,
}

impl Texture {
    pub fn new(
        gpu: &Gpu,
        shape: Shape2,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
    ) -> Self {
        let tex = gpu.device().create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("texture")),
            size: wgpu::Extent3d {
                width: shape.cols(),
                height: shape.rows(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        });

        let tex_view = tex.create_view(&wgpu::TextureViewDescriptor::default());

        Self { shape, format, usage, tex, view: tex_view }
    }

    pub fn reshape(&mut self, gpu: &Gpu, shape: Shape2) {
        if self.shape == shape {
            return;
        }
        self.shape = shape;

        self.tex.destroy();

        self.tex = gpu.device().create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("texture")),
            size: wgpu::Extent3d {
                width: self.shape.cols(),
                height: self.shape.rows(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.format,
            usage: self.usage,
            view_formats: &[],
        });

        self.view = self.tex.create_view(&wgpu::TextureViewDescriptor::default());
    }

    pub fn shape(&self) -> Shape2 {
        self.shape
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub fn copy_from(&self, encoder: &mut wgpu::CommandEncoder, src: &Texture) {
        assert_eq!(src.shape(), self.shape());

        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfoBase {
                texture: &src.tex,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfoBase {
                texture: &self.tex,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: self.shape.cols(),
                height: self.shape.rows(),
                depth_or_array_layers: 1,
            },
        );
    }

    pub fn write(&self, gpu: &Gpu, image: &RgbaImage) {
        assert_eq!(self.shape, image.shape());
        assert_eq!(self.format, wgpu::TextureFormat::Rgba8Unorm);

        gpu.queue().write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.tex,
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
                width: self.shape.cols(),
                height: self.shape.rows(),
                depth_or_array_layers: 1,
            },
        );
    }
}

impl ops::Deref for Texture {
    type Target = wgpu::TextureView;

    fn deref(&self) -> &Self::Target {
        &self.view
    }
}
