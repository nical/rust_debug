use std::f32::NAN;

use crate::{Color, Counter, Layer, Orientation, Overlay, OverlayItem, Point, FRONT_LAYER};

pub struct Graph<'a> {
    pub color: Color,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub counter: &'a Counter,
    pub reference_value: f32,
    pub orientation: Orientation,
}

impl<'a> OverlayItem for Graph<'a> {
    fn draw(&self, origin: Point, overlay: &mut Overlay) -> (Point, Point) {
        let w = self.width.unwrap_or_else(|| {
            let widget = overlay.current_group_width();
            if widget > 0 {
                widget
            } else {
                100
            }
        });
        let h = self.height.unwrap_or_else(|| {
            let widget = overlay.current_group_height();
            if widget > 0 {
                widget
            } else {
                100
            }
        });

        let rect = (
            origin,
            Point {
                x: origin.x + w,
                y: origin.y + h,
            },
        );

        draw_graph(
            FRONT_LAYER,
            rect,
            self.counter,
            self.reference_value,
            self.color,
            self.orientation,
            overlay,
        );

        rect
    }
}

pub struct Graphs<'a> {
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub counters: &'a [&'a Counter],
    pub reference_value: f32,
    pub orientation: Orientation,
}

impl<'a> OverlayItem for Graphs<'a> {
    fn draw(&self, origin: Point, overlay: &mut Overlay) -> (Point, Point) {
        let w = self.width.unwrap_or_else(|| {
            let widget = overlay.current_group_width();
            if widget > 0 {
                widget
            } else {
                100
            }
        });
        let h = self.height.unwrap_or_else(|| {
            let widget = overlay.current_group_height();
            if widget > 0 {
                widget
            } else {
                100
            }
        });

        let rect = (
            origin,
            Point {
                x: origin.x + w,
                y: origin.y + h,
            },
        );

        draw_graphs(
            FRONT_LAYER,
            rect,
            self.counters,
            self.reference_value,
            self.orientation,
            overlay,
        );

        rect
    }
}

pub struct GraphStats {
    pub avg: f32,
    pub min: f32,
    pub max: f32,
    pub samples_active: u32,
    pub samples_total: u32,
}

pub(crate) fn draw_graph(
    layer: Layer,
    rect: (Point, Point),
    counter: &Counter,
    reference_value: f32,
    color: Color,
    orientation: Orientation,
    overlay: &mut Overlay,
) -> GraphStats {
    if counter.history().is_none() {
        return GraphStats {
            avg: NAN,
            min: NAN,
            max: NAN,
            samples_active: 0,
            samples_total: 0,
        };
    }

    let rect = if orientation == Orientation::Horizontal {
        (
            Point {
                x: rect.0.y,
                y: rect.0.x,
            },
            Point {
                x: rect.1.y,
                y: rect.1.x,
            },
        )
    } else {
        rect
    };

    let mut max = std::f32::MIN;
    let mut min = std::f32::MAX;
    let mut sum = 0.0;
    let mut total_count = 0;
    let mut sample_count = 0;
    for val in counter.history().unwrap() {
        total_count += 1;
        let Some(val) = val else {
            continue;
        };
        sample_count += 1;
        max = max.max(val);
        min = min.min(val);
        sum += val;
    }

    if sample_count == 0 {
        return GraphStats {
            avg: NAN,
            min: NAN,
            max: NAN,
            samples_active: 0,
            samples_total: 0,
        };
    }

    let avg = if sample_count > 0 {
        sum / sample_count as f32
    } else {
        NAN
    };

    let w = ((rect.1.x - rect.0.x) as f32 / total_count as f32).max(1.0) as i32;
    let y_scale = (rect.1.y - rect.0.y) as f32 / max.max(reference_value);

    let mut x0 = rect.0.x;
    let y0 = rect.1.y;
    for val in counter.history().unwrap() {
        let x1 = x0 + w;
        if let Some(val) = val {
            let y1 = (y0 as f32 - val * y_scale) as i32;
            let rect = if orientation == Orientation::Horizontal {
                (Point { x: y0, y: x0 }, Point { x: y1, y: x1 })
            } else {
                (Point { x: x0, y: y0 }, Point { x: x1, y: y1 })
            };
            overlay.geometry.push_rectangle(layer, &rect, color, color);
        }
        x0 = x1;
    }

    GraphStats {
        max,
        min,
        avg,
        samples_active: sample_count,
        samples_total: total_count,
    }
}

pub(crate) fn draw_graphs(
    layer: Layer,
    rect: (Point, Point),
    counters: &[&Counter],
    reference_value: f32,
    orientation: Orientation,
    overlay: &mut Overlay,
) {
    let rect = if orientation == Orientation::Horizontal {
        (
            Point {
                x: rect.0.y,
                y: rect.0.x,
            },
            Point {
                x: rect.1.y,
                y: rect.1.x,
            },
        )
    } else {
        rect
    };

    let mut max = std::f32::MIN;
    let mut total_count = 0;

    let mut iters = Vec::with_capacity(counters.len());
    for counter in counters {
        if let Some(it) = counter.history() {
            iters.push((it, counter.descriptor.color))
        }
    }

    'outer: loop {
        let mut sample_sum = 0.0;
        for iter in &mut iters {
            let Some(val) = iter.0.next() else {
                break 'outer;
            };
            let Some(val) = val else {
                continue;
            };
            sample_sum += val;
        }
        total_count += 1;
        max = max.max(sample_sum);
    }

    iters.clear();
    for counter in counters {
        if let Some(it) = counter.history() {
            iters.push((it, counter.descriptor.color))
        }
    }

    let w = ((rect.1.x - rect.0.x) as f32 / total_count as f32).max(1.0) as i32;
    let y_scale = (rect.1.y - rect.0.y) as f32 / max.max(reference_value);

    let mut x0 = rect.0.x;

    'outer: loop {
        let mut y0 = rect.1.y;
        let x1 = x0 + w;
        for iter in &mut iters {
            let Some(val) = iter.0.next() else {
                break 'outer;
            };
            if let Some(val) = val {
                let color = iter.1;
                let y1 = (y0 as f32 - val * y_scale) as i32;
                let rect = if orientation == Orientation::Horizontal {
                    ((y0, x0).into(), (y1, x1).into())
                } else {
                    ((x0, y0).into(), (x1, y1).into())
                };
                overlay.geometry.push_rectangle(layer, &rect, color, color);
                y0 = y1;
            }
        }
        x0 = x1;
    }
}
