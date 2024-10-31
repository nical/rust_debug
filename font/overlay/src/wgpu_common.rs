use crate::embedded_font::ATLAS_WIDTH;

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

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ShaderGlobals {
    pub target_size: (f32, f32),
    pub scale: f32,
    pub opacity: f32,
    pub y_flip: f32,
}

pub fn shader_src() -> String {
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
