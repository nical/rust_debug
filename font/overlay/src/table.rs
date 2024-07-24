use crate::{
    graph::draw_graph, Color, Counter, Format, Orientation, Overlay, OverlayItem, Point,
    FONT_HEIGHT, FRONT_LAYER,
};
use std::fmt::Write;

pub struct Column {
    kind: ColumnKind,
    unit: bool,
    label: Option<&'static str>,
}

impl Column {
    const fn default() -> Self {
        Column {
            kind: ColumnKind::Empty,
            label: None,
            unit: false,
        }
    }
    pub const fn color() -> Self {
        Column {
            kind: ColumnKind::Color,
            ..Self::default()
        }
    }
    pub const fn name() -> Self {
        Column {
            kind: ColumnKind::Name,
            ..Self::default()
        }
    }
    pub const fn avg() -> Self {
        Column {
            kind: ColumnKind::Avg,
            ..Self::default()
        }
    }
    pub const fn min() -> Self {
        Column {
            kind: ColumnKind::Min,
            ..Self::default()
        }
    }
    pub const fn max() -> Self {
        Column {
            kind: ColumnKind::Max,
            ..Self::default()
        }
    }
    pub const fn value() -> Self {
        Column {
            kind: ColumnKind::Value,
            ..Self::default()
        }
    }
    pub const fn history_graph() -> Self {
        Column {
            kind: ColumnKind::HistoryGraph,
            ..Self::default()
        }
    }
    pub const fn with_unit(mut self) -> Self {
        self.unit = true;
        self
    }
    pub const fn label(mut self, label: &'static str) -> Self {
        self.label = Some(label);
        self
    }
}

#[derive(Clone, Debug)]
pub enum ColumnKind {
    Empty,
    Color,
    Name,
    Avg,
    Min,
    Max,
    Value,
    HistoryGraph,
    Changed,
}

pub struct Table<'a> {
    pub columns: &'a [Column],
    pub rows: &'a [&'a Counter],
    pub labels: bool,
}

impl<'a> OverlayItem for Table<'a> {
    fn draw(&self, origin: Point, overlay: &mut Overlay) -> (Point, Point) {
        let mut min = origin;
        let mut max = origin;

        let margin = overlay.style.margin;
        let row_height = overlay.style.line_spacing + FONT_HEIGHT as i32;

        let y0 = origin.y + FONT_HEIGHT as i32;
        let mut x = origin.x;

        for column in self.columns {
            let mut y = y0;
            let mut color_idx = 0;

            if self.labels {
                if let Some(label) = column.label {
                    let r = overlay.geometry.push_text(
                        FRONT_LAYER,
                        label,
                        Point { x, y },
                        overlay.style.title_color,
                    );
                    add_point_to_rect(r.1, &mut min, &mut max);
                }
                y += row_height + margin;
            }

            for row in self.rows {
                overlay.string_buffer.clear();

                let highlight = row
                    .descriptor
                    .safe_range
                    .as_ref()
                    .map(|range| row.displayed_max > range.end || row.displayed_min < range.start)
                    .unwrap_or(false);
                let color = if highlight {
                    overlay.style.highlight_color
                } else {
                    overlay.style.text_color[color_idx]
                };

                let r = draw_cell(x, y, column, row, color, overlay);
                add_point_to_rect(r.1, &mut min, &mut max);

                y += row_height;
                color_idx = (color_idx + 1) % 2;
            }
            let min_column_width = 0;
            let dx = (max.x - x).max(min_column_width) + overlay.style.column_spacing;
            x += dx;
        }

        (min, max)
    }
}

fn draw_cell(
    x: i32,
    y: i32,
    column: &Column,
    counter: &Counter,
    color: Color,
    overlay: &mut Overlay,
) -> (Point, Point) {
    match column.kind {
        ColumnKind::Empty => rect(((x, y), (x, y))),
        ColumnKind::Name => draw_cell_text(
            x,
            y,
            counter.descriptor.name,
            if column.unit {
                counter.descriptor.unit
            } else {
                ""
            },
            color,
            overlay,
        ),
        ColumnKind::Value => draw_cell_value(
            x,
            y,
            counter.last_value,
            counter,
            column.unit,
            color,
            overlay,
        ),
        ColumnKind::Avg => draw_cell_value(
            x,
            y,
            counter.displayed_avg,
            counter,
            column.unit,
            color,
            overlay,
        ),
        ColumnKind::Min => draw_cell_value(
            x,
            y,
            counter.displayed_min,
            counter,
            column.unit,
            color,
            overlay,
        ),
        ColumnKind::Max => draw_cell_value(
            x,
            y,
            counter.displayed_max,
            counter,
            column.unit,
            color,
            overlay,
        ),
        ColumnKind::HistoryGraph => {
            if !counter.history.is_empty() {
                let w = counter.history.len() as i32;
                let rect = (
                    Point {
                        x,
                        y: y - FONT_HEIGHT as i32,
                    },
                    Point { x: x + w, y },
                );
                let ref_value = if counter.descriptor.unit == "ms" {
                    8.0
                } else {
                    0.0
                };
                draw_graph(
                    FRONT_LAYER,
                    rect,
                    counter,
                    ref_value,
                    color,
                    Orientation::Vertical,
                    overlay,
                );

                rect
            } else {
                (Point { x, y }, Point { x, y })
            }
        }
        ColumnKind::Color => {
            let r = rect(((x, y - 11), (x + 10, y - 1)));
            let c = counter.descriptor.color;
            overlay.geometry.push_rectangle(FRONT_LAYER, &r, c, c);
            r
        }
        ColumnKind::Changed => {
            // TODO
            (Point { x, y }, Point { x, y })
        }
    }
}

fn draw_cell_text(
    x: i32,
    y: i32,
    text: &str,
    unit: &str,
    color: Color,
    overlay: &mut Overlay,
) -> (Point, Point) {
    let s = if !unit.is_empty() {
        let _ = write!(&mut overlay.string_buffer, "{text} ({unit})");
        overlay.string_buffer.as_str()
    } else {
        text
    };

    overlay
        .geometry
        .push_text(FRONT_LAYER, s, Point { x, y }, color)
}

fn draw_cell_value(
    x: i32,
    y: i32,
    val: f32,
    counter: &Counter,
    unit: bool,
    color: Color,
    overlay: &mut Overlay,
) -> (Point, Point) {
    if !val.is_finite() {
        return (Point { x, y }, Point { x, y });
    }

    let unit_str = if unit { counter.descriptor.unit } else { "" };
    overlay.string_buffer.clear();
    let _ = match counter.descriptor.format {
        Format::Int => write!(overlay.string_buffer, "{val:>5}{unit_str}"),
        Format::Float => write!(overlay.string_buffer, "{val:>5.2}{unit_str}"),
    };

    overlay
        .geometry
        .push_text(FRONT_LAYER, &overlay.string_buffer, Point { x, y }, color)
}

fn add_point_to_rect(pos: Point, min: &mut Point, max: &mut Point) {
    min.x = min.x.min(pos.x);
    min.y = min.y.min(pos.y);
    max.x = max.x.max(pos.x);
    max.y = max.y.max(pos.y);
}

fn rect(r: ((i32, i32), (i32, i32))) -> (Point, Point) {
    (
        Point {
            x: r.0 .0,
            y: r.0 .1,
        },
        Point {
            x: r.1 .0,
            y: r.1 .1,
        },
    )
}
