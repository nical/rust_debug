use std::collections::HashMap;
use std::f32::NAN;
use std::fmt::Write;

use crate::{Color, Overlay, Layer, Position, FONT_HEIGHT};

pub const BACKGROUND_LAYER: Layer = 0;
pub const FRONT_LAYER: Layer = 1;

pub enum Format {
    Int,
    Float,
}

pub struct Counter {
    sum: f32,
    samples: f32,
    min: f32,
    max: f32,
    displayed_avg: f32,
    displayed_min: f32,
    displayed_max: f32,
    descriptor: CounterDescriptor,
}

pub struct CounterDescriptor {
    pub name: &'static str,
    pub unit: &'static str,
    pub format: Format,
}

impl Counter {
    pub fn new(descritpor: CounterDescriptor) -> Self {
        Counter {
            sum: 0.0,
            samples: 0.0,
            min: 0.0,
            max: 0.0,
            displayed_avg: NAN,
            displayed_min: NAN,
            displayed_max: NAN,
            descriptor: descritpor,
        }
    }

    pub fn int(name: &'static str, unit: &'static str) -> Self {
        Counter::new(CounterDescriptor {
            name,
            unit,
            format: Format::Int,
        })
    }

    pub fn float(name: &'static str, unit: &'static str) -> Self {
        Counter::new(CounterDescriptor {
            name,
            unit,
            format: Format::Float,
        })
    }

    pub fn set(&mut self, value: f32) {
        self.samples += 1.0;
        self.sum += value;
        self.min = self.min.min(value);
        self.max = self.max.max(value);
    }

    pub fn update(&mut self) {
        if self.samples > 0.0 {
            self.displayed_avg = self.sum / self.samples;
            self.displayed_max = self.max;
            self.displayed_min = self.min;
        } else {
            self.displayed_avg = NAN;
            self.displayed_max = NAN;
            self.displayed_min = NAN;
        }
        self.samples = 0.0;
        self.sum = 0.0;
        self.min = std::f32::MAX;
        self.max = std::f32::MIN;
    }

    pub fn name(&self) -> &'static str {
        self.descriptor.name
    }
}

impl Row for Counter {
    fn get(&self, key: &str, output: &mut String) {
        if key == "name" {
            let _ = write!(output, "{}", self.descriptor. name);
            return;
        }

        let value = match key {
            "avg" if self.displayed_avg.is_finite() => Some(self.displayed_avg),
            "min" if self.displayed_max.is_finite() => Some(self.displayed_min),
            "max" if self.displayed_max.is_finite() => Some(self.displayed_max),
            _ => None,
        };

        if let Some(value) = value {
            let unit = self.descriptor.unit;
            let _ = match self.descriptor.format {
                Format::Int => write!(output, "{value:>5}{unit}"),
                Format::Float => write!(output, "{value:>5.2}{unit}"),
            };
        }
    }
}


pub trait Row {
    fn get(&self, key: &str, output: &mut String);
    fn highlight(&self) -> bool { false }
}

impl Row for HashMap<&'static str, &'static str> {
    fn get(&self, key: &str, output: &mut String) {
        if let Some(val) = self.get(key) {
            write!(output, "{val}").unwrap();
            return;
        }
    }
}

pub struct Column<'l> {
    label: &'l str,
    key: &'l str,
    min_width: f32,
}

impl<'l> Column<'l> {
    pub fn new(key: &'l str) -> Self {
        Column { label: key, key, min_width: 10.0 }
    }

    pub fn label(self, label: &'l str) -> Self {
        Column {
            label,
            key: self.key,
            min_width: self.min_width,
        }
    }

    pub fn min_width(self, min_width: u32) -> Self {
        Column {
            label: self.label,
            key: self.key,
            min_width: min_width as f32,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Style {
    pub margin: f32,
    pub line_spacing: f32,
    pub column_spacing: f32,
    pub min_column_width: f32,
    pub min_table_width: f32,
    pub backgroun_gradient: [Color; 2],
    pub text_color: [Color; 2],
    pub title_color: Color,
    pub highlight_color: Color,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            margin: 10.0,
            line_spacing: 2.0,
            column_spacing: 20.0,
            min_column_width: 50.0,
            min_table_width: 350.0,
            backgroun_gradient: [
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

pub struct Layout {
    style: Style,
    string_buffer: String,
}

impl Layout {
    pub fn new(style: Style) -> Self {
        Layout {
            style,
            string_buffer: String::new(),
        }
    }

    pub fn draw_table(
        &mut self,
        mut origin: Position,
        columns: &[Column],
        rows: &[&dyn Row],
        output: &mut Overlay
    ) -> (Position, Position) {
        origin.0 += self.style.margin;
        origin.1 += self.style.margin;

        let mut min = origin;
        let mut max = origin;

        let margin = self.style.margin;
        let row_height = self.style.line_spacing + FONT_HEIGHT as f32;

        let y0 = origin.1 + FONT_HEIGHT as f32;
        let mut x = origin.0;

        for column in columns {
            let r = output.push_text(FRONT_LAYER, column.label, (x, y0), self.style.title_color);
            add_point_to_rect(r.1, &mut min, &mut max);

            let mut y = y0 + row_height + margin;
            let mut color_idx = 0;
            for row in rows {
                self.string_buffer.clear();
                row.get(column.key, &mut self.string_buffer);
                let color = if row.highlight() { self.style.highlight_color } else { self.style.text_color[color_idx] };

                let r = output.push_text(FRONT_LAYER, &self.string_buffer, (x, y), color);
                add_point_to_rect(r.1, &mut min, &mut max);

                y = r.1.1 + row_height;
                color_idx = (color_idx + 1) % 2;
            }
            let min_column_width = self.style.min_column_width.max(column.min_width);
            let dx = (max.0 - x).max(min_column_width) + self.style.column_spacing;
            x += dx;
        }

        let mut bg = inflate_rect((min, max), self.style.margin);
        bg.1.0 = bg.0.0 + self.style.min_table_width.max(rect_width(bg));

        output.push_rectangle(
            BACKGROUND_LAYER,
            &bg,
            self.style.backgroun_gradient[0],
            self.style.backgroun_gradient[1],
        );

        add_point_to_rect(bg.0, &mut min, &mut max);
        add_point_to_rect(bg.1, &mut min, &mut max);

        (min, max)
    }

    pub fn draw_text(
        &mut self,
        origin: Position,
        text: &str,
        output: &mut Overlay,
    ) -> (Position, Position) {
        let p = (
            origin.0 + self.style.margin,
            origin.1 + self.style.margin + FONT_HEIGHT as f32
        );

        let rect = output.push_text(
            FRONT_LAYER,
            text,
            p,
            self.style.text_color[0],
        );

        let mut background = inflate_rect(rect, self.style.margin);
        add_point_to_rect(origin, &mut background.0, &mut background.1);
        output.push_rectangle(
            BACKGROUND_LAYER,
            &background,
            self.style.backgroun_gradient[0],
            self.style.backgroun_gradient[1],
        );

        background
    }
}

fn rect_width(r: (Position, Position)) -> f32 {
    r.1.0 - r.0.0
}

fn inflate_rect(mut r: (Position, Position), amount: f32) -> (Position, Position) {
    r.0.0 -= amount;
    r.0.1 -= amount;
    r.1.0 += amount;
    r.1.1 += amount;

    r
}

fn add_point_to_rect(pos: Position, min: &mut Position, max: &mut Position) {
    min.0 = min.0.min(pos.0);
    min.1 = min.1.min(pos.1);
    max.0 = max.0.max(pos.0);
    max.1 = max.1.max(pos.1);
}
