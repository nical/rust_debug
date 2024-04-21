//! A basic low-overhead debugging overlay for use with GPU APIs such as `wgpu`.
//!
//! # Features
//!
//! Enable one or several or the builtin runderers using the following cargo features:
//! - `wgpu`
//! - `wgpu-core` (TODO)
//!

pub mod embedded_font;
pub mod table;
pub mod graph;
mod counter;
#[cfg(feature="wgpu")] pub mod wgpu;

use embedded_font::*;
use bytemuck::{Pod, Zeroable};

pub use counter::*;

pub const BACKGROUND_LAYER: Layer = 0;
pub const FRONT_LAYER: Layer = 1;

/// A 2D position (in pixels).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Point { pub x: i32, pub y: i32 }

impl From<(f32, f32)> for Point {
    fn from(val: (f32, f32)) -> Self {
        Point { x: val.0 as i32, y: val.1 as i32 }
    }
}

impl From<(i32, i32)> for Point {
    fn from(val: (i32, i32)) -> Self {
        Point { x: val.0, y: val.1 }
    }
}

/// A 2D position (in pixels).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PointF { pub x: f32, pub y: f32 }

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

pub struct OverlayGeometry {
    vertices: Vec<Vertex>,
    layers: Vec<LayerGeometry>,
}

impl OverlayGeometry {
    pub fn new(layer_count: usize) -> Self {
        let mut layers = Vec::new();
        for _ in 0..layer_count {
            layers.push(LayerGeometry {
                indices: Vec::new(),
            });
        }
        OverlayGeometry {
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
        mut position: Point,
        color: Color,
    ) -> (Point, Point) {
        let color = color_to_u32(color);
        let mut min = position;
        let mut max = min;

        for c in text.chars() {
            if c == '\n' {
                position.x = min.x;
                position.y += FONT_HEIGHT as i32;
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

            let x0 = position.x + glyph.offset.0 as i32;
            let y0 = position.y + glyph.offset.1 as i32;
            let x1 = x0 + (glyph.uv1.0 - glyph.uv0.0) as i32;
            let y1 = y0 + (glyph.uv1.1 - glyph.uv0.1) as i32;

            let offset = self.vertices.len() as u16;
            self.vertices.push(Vertex { x: x0 as f32, y: y0 as f32, uv: uv0x|uv0y, color });
            self.vertices.push(Vertex { x: x1 as f32, y: y0 as f32, uv: uv1x|uv0y, color });
            self.vertices.push(Vertex { x: x1 as f32, y: y1 as f32, uv: uv1x|uv1y, color });
            self.vertices.push(Vertex { x: x0 as f32, y: y1 as f32, uv: uv0x|uv1y, color });
            let layer = &mut self.layers[layer];
            for i in [0u16, 1, 2, 0, 2, 3] {
                layer.indices.push(offset + i);
            }

            position.x += glyph.x_advance as i32;

            min.x = min.x.min(x0);
            min.y = min.y.min(y0);
            max.x = max.x.max(x1);
            max.y = max.y.max(y1);
        }

        (min, max)
    }

    pub fn push_rectangle(
        &mut self,
        layer: Layer,
        rect: &(Point, Point),
        color0: Color,
        color1: Color,
    ) {
        let uv = (OPAQUE_PIXEL.0 as u32) << 16 | OPAQUE_PIXEL.1 as u32;
        let x0 = rect.0.x;
        let y0 = rect.0.y;
        let x1 = rect.1.x;
        let y1 = rect.1.y;
        let color0 = color_to_u32(color0);
        let color1 = color_to_u32(color1);

        let offset = self.vertices.len() as u16;
        self.vertices.push(Vertex { x: x0 as f32, y: y0 as f32, uv, color: color0 });
        self.vertices.push(Vertex { x: x1 as f32, y: y0 as f32, uv, color: color0 });
        self.vertices.push(Vertex { x: x1 as f32, y: y1 as f32, uv, color: color1 });
        self.vertices.push(Vertex { x: x0 as f32, y: y1 as f32, uv, color: color1 });
        let layer = &mut self.layers[layer];
        for i in [0u16, 1, 2, 0, 2, 3] {
            layer.indices.push(offset + i);
        }
    }

    pub fn push_mesh(
        &mut self,
        layer: Layer,
        vertices: &[PointF],
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
                x: vertex.x,
                y: vertex.y,
                uv,
                color,
            });
        }
        for idx in indices {
            layer.indices.push(offset + *idx);
        }
    }
}

pub struct Overlay {
    pub geometry: OverlayGeometry,
    pub style: Style,
    pub cursor: Point,
    pub item_flow: Orientation,
    pub group_flow: Orientation,
    pub string_buffer: String,
    group_area: (Point, Point),
    in_group: bool,
    max_x: i32,
    max_y: i32,
}

impl Overlay {
    pub fn new() -> Self {
        let style = Style::default();
        let cursor = Point { x: style.margin, y: style.margin };
        Overlay {
            geometry: OverlayGeometry::new(2),
            style,
            cursor,
            item_flow: Orientation::Horizontal,
            group_flow: Orientation::Vertical,
            string_buffer: String::with_capacity(128),
            group_area: (cursor, cursor),
            in_group: false,
            max_x: 0,
            max_y: 0,
        }
    }

    pub fn begin_frame(&mut self) {
        self.geometry.begin_frame();

        self.cursor = Point { x: self.style.margin, y: self.style.margin };
        self.group_area = (self.cursor, self.cursor);
        self.max_x = 0;
        self.max_y = 0;
        self.in_group = false;
    }

    pub fn current_group_width(&self) -> i32 {
        self.group_area.1.x - self.group_area.0.x
    }

    pub fn current_group_height(&self) -> i32 {
        self.group_area.1.y - self.group_area.0.y
    }

    pub fn draw_item(&mut self, item: &dyn OverlayItem) {
        let first = !self.in_group;
        if !self.in_group {
            self.begin_group();
        }

        let margin = if first { 0 } else { self.style.margin };
        self.cursor = match self.item_flow {
            Orientation::Vertical => Point {
                x: self.group_area.0.x,
                y: self.group_area.1.y + margin,
            },
            Orientation::Horizontal => Point {
                x: self.group_area.1.x + margin,
                y: self.group_area.0.y,
            },
        };

        let rect = item.draw(self.cursor, self);

        self.group_area.0.x = self.group_area.0.x.min(rect.0.x);
        self.group_area.0.y = self.group_area.0.y.min(rect.0.y);
        self.group_area.1.x = self.group_area.1.x.max(rect.1.x);
        self.group_area.1.y = self.group_area.1.y.max(rect.1.y);
    }


    pub fn push_separator(&mut self) {
        if !self.in_group {
            return;
        }

        match self.item_flow {
            Orientation::Vertical => {
                self.cursor.y += self.style.margin * 3;
            }
            Orientation::Horizontal => {
                self.cursor.x += self.style.margin * 3;
            }
        }
    }

    pub fn push_column(&mut self) {
        if self.in_group {
            self.end_group();
        }

        let p = Point {
            x: self.max_x + self.style.margin * 3,
            y: self.style.margin,
        };

        self.group_area = (p, p);
    }

    fn begin_group(&mut self) {
        match self.group_flow {
            Orientation::Vertical => {
                let margin = if self.group_area.1.y > self.style.margin {
                    self.style.margin * 3
                } else {
                    0
                };
                self.cursor.x = self.group_area.0.x;
                self.cursor.y = self.group_area.1.y + margin;
            }
            Orientation::Horizontal => {
                let margin = if self.group_area.1.x > self.style.margin {
                    self.style.margin * 3
                } else {
                    0
                };
                self.cursor.x = self.group_area.1.x + margin;
                self.cursor.y = self.group_area.0.y;
            }
        }

        self.group_area = (self.cursor, self.cursor);
        self.in_group = true;
    }

    pub fn end_group(&mut self) {
        self.in_group = false;
        if self.group_area.0.x >= self.group_area.1.x
        || self.group_area.0.y >= self.group_area.1.y {
            return;
        }

        self.group_area.1.x = self.group_area.1.x.max(self.group_area.0.x + self.style.min_group_width);
        self.group_area.1.y = self.group_area.1.y.max(self.group_area.0.y + self.style.min_group_height);

        self.max_x = self.max_x.max(self.group_area.1.x);
        self.max_y = self.max_y.max(self.group_area.1.y);

        let margin = self.style.margin;
        let mut bg = self.group_area;
        bg.0.x -= margin;
        bg.0.y -= margin;
        bg.1.x += margin;
        bg.1.y += margin;

        self.geometry.push_rectangle(
            BACKGROUND_LAYER,
            &bg,
            self.style.background[0],
            self.style.background[1],
        );
    }

    pub fn finish(&mut self) {
        if self.in_group {
            self.end_group();
        }
    }
}

pub trait OverlayItem {
    fn draw(&self, position: Point, output: &mut Overlay) -> (Point, Point);
}

impl<'a> OverlayItem for &'a str {
    fn draw(&self, position: Point, output: &mut Overlay) -> (Point, Point) {
        let p = Point {
            x: position.x,
            y: position.y + FONT_HEIGHT as i32
        };

        output.geometry.push_text(
            FRONT_LAYER,
            self,
            p,
            output.style.text_color[0],
        )
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Style {
    pub margin: i32,
    pub line_spacing: i32,
    pub min_group_width: i32,
    pub min_group_height: i32,
    pub column_spacing: i32,
    pub background: [Color; 2],
    pub text_color: [Color; 2],
    pub title_color: Color,
    pub highlight_color: Color,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            margin: 10,
            line_spacing: 2,
            min_group_width: 0,
            min_group_height: 0,
            column_spacing: 20,
            background: [
                (0, 0, 0, 255),
                (0, 0, 0, 200)
            ],
            text_color: [
                (255, 255, 255, 255),
                (200, 200, 200, 255),
            ],
            title_color: (120, 150, 255, 255),
            highlight_color: (255, 100, 100, 255),
        }
    }
}
