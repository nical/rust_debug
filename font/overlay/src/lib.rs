//! A basic low-overhead debugging overlay for use with GPU APIs such as `wgpu`.
//!
//! # Features
//!
//! Enable one or several or the builtin runderers using the following cargo features:
//! - `wgpu`
//! - `wgpu-core` (TODO)
//!

pub mod embedded_font;
pub mod views;
#[cfg(features="wgpu")] pub mod wgpu;

use embedded_font::*;
use bytemuck::{Pod, Zeroable};

/// A 2D position (in pixels).
pub type Position = (f32, f32);
/// An 8-bit per channel RGBA color value.
pub type Color = (u8, u8, u8, u8);
/// The index of an overlay layer.
pub type Layer = usize;

fn color_to_u32(color: Color) -> u32 {
    (color.0 as u32) << 24
    | (color.1 as u32) << 16
    | (color.2 as u32) << 8
    | color.3 as u32
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub uv: u32,
    pub color: u32,
}

unsafe impl Pod for Vertex {}
unsafe impl Zeroable for Vertex {}

pub(crate) struct LayerGeometry {
    pub indices: Vec<u16>,
}

pub struct Overlay {
    vertices: Vec<Vertex>,
    layers: Vec<LayerGeometry>,
}

impl Overlay {
    pub fn new(layer_count: usize) -> Self {
        let mut layers = Vec::new();
        for _ in 0..layer_count {
            layers.push(LayerGeometry {
                indices: Vec::new(),
            });
        }
        Overlay {
            vertices: Vec::new(),
            layers,
        }
    }

    pub fn begin_frame(&mut self) {
        self.vertices.clear();
        for layer in &mut self.layers {
            layer.indices.clear();
        }
    }

    pub fn push_text(
        &mut self,
        layer: Layer,
        text: &str,
        mut position: Position,
        color: Color,
    ) -> (Position, Position) {
        let color = color_to_u32(color);
        let mut min = position;
        let mut max = min;

        for c in text.chars() {
            if c == '\n' {
                position.0 = min.0;
                position.1 += FONT_HEIGHT as f32;
                continue;
            }

            let idx = c as usize - FIRST_CHAR as usize;
            if idx >= GLYPH_INFO.len() {
                continue;
            }
            let glyph = &GLYPH_INFO[idx];

            let uv0x = (glyph.uv0.0 as u32) << 16;
            let uv0y = glyph.uv0.1 as u32;
            let uv1x = (glyph.uv1.0 as u32) << 16;
            let uv1y = glyph.uv1.1 as u32;

            let x0 = position.0 + glyph.offset.0 as f32;
            let y0 = position.1 + glyph.offset.1 as f32;
            let x1 = x0 + (glyph.uv1.0 - glyph.uv0.0) as f32;
            let y1 = y0 + (glyph.uv1.1 - glyph.uv0.1) as f32;

            let offset = self.vertices.len() as u16;
            self.vertices.push(Vertex { x: x0, y: y0, uv: uv0x|uv0y, color });
            self.vertices.push(Vertex { x: x1, y: y0, uv: uv1x|uv0y, color });
            self.vertices.push(Vertex { x: x1, y: y1, uv: uv1x|uv1y, color });
            self.vertices.push(Vertex { x: x0, y: y1, uv: uv0x|uv1y, color });
            let layer = &mut self.layers[layer];
            for i in [0u16, 1, 2, 0, 2, 3] {
                layer.indices.push(offset + i);
            }

            position.0 += glyph.x_advance;

            min.0 = min.0.min(x0);
            min.1 = min.1.min(y0);
            max.0 = max.0.max(x1);
            max.1 = max.1.max(y1);
        }

        (min, max)
    }

    pub fn push_rectangle(
        &mut self,
        layer: Layer,
        rect: &(Position, Position),
        color0: Color,
        color1: Color,
    ) {
        let uv = (OPAQUE_PIXEL.0 as u32) << 16 | OPAQUE_PIXEL.1 as u32;
        let x0 = rect.0.0;
        let y0 = rect.0.1;
        let x1 = rect.1.0;
        let y1 = rect.1.1;
        let color0 = color_to_u32(color0);
        let color1 = color_to_u32(color1);

        let offset = self.vertices.len() as u16;
        self.vertices.push(Vertex { x: x0, y: y0, uv, color: color0 });
        self.vertices.push(Vertex { x: x1, y: y0, uv, color: color0 });
        self.vertices.push(Vertex { x: x1, y: y1, uv, color: color1 });
        self.vertices.push(Vertex { x: x0, y: y1, uv, color: color1 });
        let layer = &mut self.layers[layer];
        for i in [0u16, 1, 2, 0, 2, 3] {
            layer.indices.push(offset + i);
        }
    }

    pub fn push_mesh(
        &mut self,
        layer: Layer,
        vertices: &[Position],
        indices: &[u16],
        color: Color,
    ) {
        let uv = (OPAQUE_PIXEL.0 as u32) << 16 | OPAQUE_PIXEL.1 as u32;
        let layer = &mut self.layers[layer];
        self.vertices.reserve(vertices.len());
        layer.indices.reserve(indices.len());
        let offset = self.vertices.len() as u16;
        let color = color_to_u32(color);
        for vertex in vertices {
            self.vertices.push(Vertex {
                x: vertex.0,
                y: vertex.1,
                uv,
                color,
            });
        }
        for idx in indices {
            layer.indices.push(offset + *idx);
        }
    }
}
