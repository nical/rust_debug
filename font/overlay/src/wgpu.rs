use std::mem::size_of;
use wgpu::{self, util::DeviceExt};

use crate::{
    embedded_font::{ATLAS_HEIGHT, ATLAS_WIDTH},
    Vertex,
};

/// Initial parameters for the overlay renderer.
#[derive(Clone, Debug)]
pub struct RendererOptions {
    /// Format of the color target.
    pub target_format: wgpu::TextureFormat,
    /// Format of the dept stencil target, if any.
    pub depth_stencil_format: Option<wgpu::TextureFormat>,
    /// Number of samples per pixel.
    pub sample_count: u32,
    /// Whether to invert the y-coordinate when displaying the overlay.
    pub y_flip: bool,
    /// Global scaling factor.
    pub scale_factor: f32,
}

impl Default for RendererOptions {
    fn default() -> Self {
        RendererOptions {
            target_format: wgpu::TextureFormat::Rgba8Unorm,
            depth_stencil_format: None,
            sample_count: 1,
            y_flip: true,
            scale_factor: 1.0,
        }
    }
}

/// Renders an overlay using `wgpu`.
pub struct Renderer {
    glyph_atlas_texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
    vbo: Option<(wgpu::Buffer, usize)>,
    ibo: Option<(wgpu::Buffer, usize)>,
    ubo: wgpu::Buffer,
    index_count: u32,
    y_flip: bool,
    scale: f32,
    globals: ShaderGlobals,
}

impl Renderer {
    /// Constructor.
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, options: &RendererOptions) -> Self {
        let width = ATLAS_WIDTH;
        let height = width;

        let glyph_atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Debug overlay atlas"),
            dimension: wgpu::TextureDimension::D2,
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            mip_level_count: 1,
            sample_count: 1,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &glyph_atlas_texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            crate::embedded_font::GLYPH_ATLAS,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(width),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width,
                height: ATLAS_HEIGHT,
                depth_or_array_layers: 1,
            },
        );

        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Debug overlay"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(32),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let glyph_atlas_view = glyph_atlas_texture.create_view(&Default::default());

        let ubo = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Debug overlay globals"),
            contents: bytemuck::cast_slice(&[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Debug overlay"),
            layout: &bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &ubo,
                        offset: 0,
                        size: wgpu::BufferSize::new(32),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&glyph_atlas_view),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Debug overlay"),
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
        });

        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Debug overlay"),
            source: wgpu::ShaderSource::Wgsl(shader_src().into()),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Debug overlay mesh"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 16,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Uint32x2,
                            offset: 8,
                            shader_location: 1,
                        },
                    ],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: options.target_format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                polygon_mode: wgpu::PolygonMode::Fill,
                front_face: wgpu::FrontFace::Ccw,
                strip_index_format: None,
                cull_mode: None,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: options
                .depth_stencil_format
                .map(|format| wgpu::DepthStencilState {
                    format,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
            multiview: None,
            multisample: wgpu::MultisampleState {
                count: options.sample_count,
                ..wgpu::MultisampleState::default()
            },
            cache: None,
        });

        Renderer {
            glyph_atlas_texture,
            bind_group,
            pipeline,

            vbo: None,
            ibo: None,
            ubo,
            index_count: 0,
            y_flip: options.y_flip,
            scale: options.scale_factor,
            globals: ShaderGlobals {
                target_size: (0.0, 0.0),
                scale: 0.0,
                opacity: 0.0,
                y_flip: 1.0,
            },
        }
    }

    /// Transfers the overlay information to the GPU.
    ///
    /// Must be called once per frame where the overlay is shown, before calling `renderer`.
    pub fn update(
        &mut self,
        overlay: &crate::OverlayGeometry,
        taregt_size: (u32, u32),
        opacity: f32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        const VTX_SIZE: usize = size_of::<Vertex>();
        const IDX_SIZE: usize = size_of::<u16>();

        let vbo_len = overlay.vertices.len();
        let ibo_len = overlay.layers.iter().map(|l| l.indices.len()).sum();

        let alloc_vbo = self
            .vbo
            .as_ref()
            .map(|(_, len)| *len <= vbo_len)
            .unwrap_or(true);
        let alloc_ibo = self
            .ibo
            .as_ref()
            .map(|(_, len)| *len <= ibo_len)
            .unwrap_or(true);

        if alloc_vbo {
            self.vbo = Some((
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Debug overlay vertices"),
                    size: (vbo_len * VTX_SIZE) as u64,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
                vbo_len,
            ));
        }

        if alloc_ibo {
            self.ibo = Some((
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Debug overlay indices"),
                    size: (ibo_len * IDX_SIZE) as u64,
                    usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
                ibo_len,
            ));
        }

        if !overlay.vertices.is_empty() {
            queue.write_buffer(
                &self.vbo.as_ref().unwrap().0,
                0,
                bytemuck::cast_slice(&overlay.vertices[..]),
            );
        }

        let mut ibo_byte_offset = 0;
        self.index_count = 0;
        for layer in &overlay.layers {
            if layer.indices.is_empty() {
                continue;
            }
            queue.write_buffer(
                &self.ibo.as_ref().unwrap().0,
                ibo_byte_offset,
                bytemuck::cast_slice(&layer.indices[..]),
            );
            ibo_byte_offset += (layer.indices.len() * IDX_SIZE) as u64;
            self.index_count += layer.indices.len() as u32;
        }

        let w = taregt_size.0 as f32;
        let h = taregt_size.1 as f32;
        let globals = ShaderGlobals {
            target_size: (w, h),
            scale: self.scale,
            opacity,
            y_flip: if self.y_flip { -1.0 } else { 1.0 },
        };

        if self.globals != globals {
            queue.write_buffer(
                &self.ubo,
                0,
                bytemuck::cast_slice(&[
                    globals.target_size.0,
                    globals.target_size.1,
                    globals.scale,
                    globals.opacity,
                    globals.y_flip,
                ]),
            );
            self.globals = globals;
        }
    }

    /// Display the overlay in a render pass.
    ///
    /// Must be called once per frame where the overlay is shown, after calling `update`.
    pub fn render<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        if self.index_count == 0 {
            return;
        }

        let vbo = &self.vbo.as_ref().unwrap().0;
        let ibo = &self.ibo.as_ref().unwrap().0;

        pass.set_vertex_buffer(0, vbo.slice(..));
        pass.set_index_buffer(ibo.slice(..), wgpu::IndexFormat::Uint16);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_pipeline(&self.pipeline);

        pass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.glyph_atlas_texture.destroy();
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct ShaderGlobals {
    target_size: (f32, f32),
    scale: f32,
    opacity: f32,
    y_flip: f32,
}

fn shader_src() -> String {
    format!(
        "
const ATLAS_SIZE: f32 = {ATLAS_WIDTH}.0;

struct Globals {{
    target_size: vec2f,
    scale: f32,
    opacity: f32,
    y_flip: f32,
}};

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var glyph_atlas: texture_2d<f32>;

struct VertexOutput {{
    @builtin(position) position: vec4f,
    @location(0) color: vec4f,
    @location(1) uv: vec2f,
}};

@vertex fn vs_main(
    @location(0) position: vec2f,
    @location(1) uv_color: vec2u,
) -> VertexOutput {{
    let uv = vec2f(
        f32(uv_color.x >> 16u),
        f32(uv_color.x & 0xFFFFu)
    );

    let color = vec4f(
        f32((uv_color.y >> 24u) & 0xFFu),
        f32((uv_color.y >> 16u) & 0xFFu),
        f32((uv_color.y >>  8u) & 0xFFu),
        f32(uv_color.y & 0xFFu) * globals.opacity,
    ) / 255.0;

    var screen_pos = ((position * globals.scale) / globals.target_size) * 2.0 - 1.0;
    screen_pos.y *= globals.y_flip;

    return VertexOutput(
        vec4f(screen_pos, 0.0, 1.0),
        color,
        uv,
    );
}}

@fragment fn fs_main(
    @location(0) color: vec4f,
    @location(1) uv: vec2f,
) -> @location(0) vec4f {{
    let texel = textureLoad(glyph_atlas, vec2u(uv), 0).r;
    return color * color.a * texel;
}}
"
    )
}
