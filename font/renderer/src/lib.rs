pub mod embedded_font;

use embedded_font::*;
//use bytemuck::{Pod, Zeroable};

pub type Position = (f32, f32);
pub type Color = (u8, u8, u8, u8);
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

pub struct LayerGeometry {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

pub struct DebugGeometry {
    layers: Vec<LayerGeometry>,
    pub scale: f32,
    pub line_spacing: f32,
}

impl DebugGeometry {
    pub fn new(layer_count: u32) -> Self {
        let mut layers = Vec::new();
        for _ in 0..layer_count {
            layers.push(LayerGeometry {
                vertices: Vec::new(),
                indices: Vec::new(),
            });
        }
        DebugGeometry {
            layers,
            scale: 1.0,
            line_spacing: 0.0,
        }
    }

    pub fn begin_frame(&mut self) {
        for layer in &mut self.layers {
            layer.vertices.clear();
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
                position.1 += FONT_HEIGHT as f32 * self.scale + self.line_spacing;
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

            let x0 = self.scale * (position.0 + glyph.offset.0 as f32);
            let y0 = self.scale * (position.1 + glyph.offset.1 as f32);
            let x1 = self.scale * (x0 + (glyph.uv1.0 - glyph.uv0.0) as f32 );
            let y1 = self.scale * (y0 + (glyph.uv1.1 - glyph.uv0.1) as f32 );

            let layer = &mut self.layers[layer];
            let offset = layer.vertices.len() as u16;
            layer.vertices.push(Vertex { x: x0, y: y0, uv: uv0x|uv0y, color });
            layer.vertices.push(Vertex { x: x1, y: y0, uv: uv1x|uv0y, color });
            layer.vertices.push(Vertex { x: x1, y: y1, uv: uv1x|uv1y, color });
            layer.vertices.push(Vertex { x: x0, y: y1, uv: uv0x|uv1y, color });
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
        let x0 = self.scale * rect.0.0;
        let y0 = self.scale * rect.0.1;
        let x1 = self.scale * rect.1.0;
        let y1 = self.scale * rect.1.1;
        let color0 = color_to_u32(color0);
        let color1 = color_to_u32(color1);

        let layer = &mut self.layers[layer];
        let offset = layer.vertices.len() as u16;
        layer.vertices.push(Vertex { x: x0, y: y0, uv, color: color0 });
        layer.vertices.push(Vertex { x: x1, y: y0, uv, color: color0 });
        layer.vertices.push(Vertex { x: x1, y: y1, uv, color: color1 });
        layer.vertices.push(Vertex { x: x0, y: y1, uv, color: color1 });
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
        layer.vertices.reserve(vertices.len());
        layer.indices.reserve(indices.len());
        let offset = layer.vertices.len() as u16;
        let color = color_to_u32(color);
        for vertex in vertices {
            layer.vertices.push(Vertex {
                x: self.scale * vertex.0,
                y: self.scale * vertex.1,
                uv,
                color,
            });
        }
        for idx in indices {
            layer.indices.push(offset + *idx);
        }
    }
}
