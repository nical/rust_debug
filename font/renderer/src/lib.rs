pub mod embedded_font;

use embedded_font::*;

pub type Position = (f32, f32);
pub type Color = (u8, u8, u8, u8);

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

pub struct DebugGeometry {
    pub quad_vertices: Vec<Vertex>,
    pub quad_indices: Vec<u16>,
    pub glyph_vertices: Vec<Vertex>,
    pub glyph_indices: Vec<u16>,
    pub scale: f32,
    pub line_spacing: f32,
}

impl DebugGeometry {
    pub fn new() -> Self {
        DebugGeometry {
            quad_vertices: Vec::new(),
            quad_indices: Vec::new(),
            glyph_vertices: Vec::new(),
            glyph_indices: Vec::new(),
            scale: 1.0,
            line_spacing: 0.0,
        }
    }

    pub fn begin_frame(&mut self) {
        self.quad_vertices.clear();
        self.quad_indices.clear();
        self.glyph_vertices.clear();
        self.glyph_indices.clear();
    }

    pub fn push_text(
        &mut self,
        text: &str,
        mut position: Position,
        color: Color,
    ) -> (Position, Position) {
        let color = color_to_u32(color);
        let min = position;
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

            let x0 = self.scale * (position.0 + glyph.offset.0) as f32;
            let y0 = self.scale * (position.1 + glyph.offset.1) as f32;
            let x1 = self.scale * (x0 + (glyph.uv1.0 - glyph.uv0.0)) as f32;
            let y1 = self.scale * (y0 + (glyph.uv1.1 - glyph.uv0.1)) as f32;

            let offset = self.glyph_vertices.len() as u16;
            self.glyph_vertices.push(Vertex { x: x0, y: y0, uv: uv0x|uv0y, color });
            self.glyph_vertices.push(Vertex { x: x1, y: y0, uv: uv1x|uv0y, color });
            self.glyph_vertices.push(Vertex { x: x1, y: y1, uv: uv1x|uv1y, color });
            self.glyph_vertices.push(Vertex { x: x0, y: y1, uv: uv0x|uv1y, color });
            for i in [0u16, 1, 2, 0, 2, 3] {
                self.glyph_indices.push(offset + i);
            }

            position.0 += glyph.x_advance;

            max.0 = max.0.max(x1);
            max.1 = max.1.max(y1);
        }

        (min, max)
    }

    pub fn push_rectangle(
        &mut self,
        rect: &(Position, Position),
        color0: Color,
        color1: Color,
    ) {
        let uv = 0;
        let x0 = self.scale * rect.0.0;
        let y0 = self.scale * rect.0.1;
        let x1 = self.scale * rect.1.0;
        let y1 = self.scale * rect.1.1;
        let color0 = color_to_u32(color0);
        let color1 = color_to_u32(color1);
        let offset = self.glyph_vertices.len() as u16;
        self.glyph_vertices.push(Vertex { x: x0, y: y0, uv, color: color0 });
        self.glyph_vertices.push(Vertex { x: x1, y: y0, uv, color: color0 });
        self.glyph_vertices.push(Vertex { x: x1, y: y1, uv, color: color1 });
        self.glyph_vertices.push(Vertex { x: x0, y: y1, uv, color: color1 });
        for i in [0u16, 1, 2, 0, 2, 3] {
            self.glyph_indices.push(offset + i);
        }
    }

    pub fn push_mesh(
        &mut self,
        veritces: &[Postion],
        indices: &[u16],
        color: Color,
    ) {
        self.quad_vertices.reserve(veritces.len());
        self.quad_indices.reserve(indices.len());
        let offset = quad_vertices.len() as u16;
        let color = color_to_u32(color);
        for vertex in vertices {
            self.quad_vertices.push(Vertex {
                x: self.scale * vertex.0,
                y: self.scale * vertex.1,
                uv: 0,
                color,
            });
        }
        for idx in indices {
            self.quad_indices.push(offset + *idx);
        }
    }
}
