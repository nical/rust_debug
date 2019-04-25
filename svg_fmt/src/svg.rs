use std::fmt;

#[derive(Copy, Clone, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl fmt::Debug for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "rgb({},{},{})", self.r, self.g, self.b)
    }
}

pub fn rgb(r: u8, g: u8, b: u8) -> Color { Color { r, g, b } }
pub fn black() -> Color { rgb(0, 0, 0) }
pub fn white() -> Color { rgb(255, 255, 255) }
pub fn red() -> Color { rgb(255, 0, 0) }
pub fn green() -> Color { rgb(0, 255, 0) }
pub fn blue() -> Color { rgb(0, 0, 255) }

#[derive(Copy, Clone, PartialEq)]
pub enum Fill {
    Color(Color),
    None,
}

#[derive(Copy, Clone, PartialEq)]
pub enum Stroke {
    Color(Color, f32),
    None,
}

impl fmt::Debug for Fill {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Fill::Color(color) => write!(f, "fill:{:?}", color),
            Fill::None => write!(f, "fill:none"),
        }
    }
}

impl fmt::Debug for Stroke {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Stroke::Color(color, radius) => write!(f, "stroke:{:?};stroke-width:{:?}", color, radius),
            Stroke::None => write!(f, "stroke:none"),
        }
    }
}

impl Into<Fill> for Color {
    fn into(self) -> Fill {
        Fill::Color(self)
    }
}

impl Into<Stroke> for Color {
    fn into(self) -> Stroke {
        Stroke::Color(self, 1.0)
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub fill: Fill,
    pub stroke: Stroke,
    pub border_radius: f32,
}

pub fn rectangle(x: f32, y: f32, w: f32, h: f32) -> Rectangle {
    Rectangle {
        x, y, w, h,
        fill: Fill::Color(black()),
        stroke: Stroke::None,
        border_radius: 0.0,
    }
}

impl Rectangle {
    pub fn fill<F>(mut self, fill: F) -> Self
    where F: Into<Fill> {
        self.fill = fill.into();
        self
    }

    pub fn stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = stroke;
        self
    }

    pub fn border_radius(mut self, r: f32) -> Self {
        self.border_radius = r;
        self
    }

    pub fn offset(mut self, dx: f32, dy: f32) -> Self {
        self.x += dx;
        self.y += dy;
        self
    }

    pub fn inflate(mut self, dx: f32, dy: f32) -> Self {
        self.x -= dx;
        self.y -= dy;
        self.w += 2.0 * dx;
        self.h += 2.0 * dy;
        self
    }
}

impl fmt::Debug for Rectangle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
            r#"<rect x="{}" y="{}" width="{}" height="{}" ry="{}" style="{:?};{:?}" />""#,
            self.x, self.y, self.w, self.h,
            self.border_radius,
            self.fill,
            self.stroke,
        )
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct Circle {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub fill: Fill,
    pub stroke: Stroke,
}

impl Circle {
    pub fn fill<F>(mut self, fill: F) -> Self
    where F: Into<Fill> {
        self.fill = fill.into();
        self
    }

    pub fn stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = stroke;
        self
    }

    pub fn offset(mut self, dx: f32, dy: f32) -> Self {
        self.x += dx;
        self.y += dy;
        self
    }

    pub fn inflate(mut self, by: f32) -> Self {
        self.radius += by;
        self
    }
}

impl fmt::Debug for Circle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
            r#"<circle cx="{}" cy="{}" r="{}" style="{:?};{:?}" />""#,
            self.x, self.y, self.radius,
            self.fill,
            self.stroke,
        )
    }
}

#[derive(Clone, PartialEq)]
pub struct Polygon {
    pub points: Vec<[f32; 2]>,
    pub fill: Fill,
    pub stroke: Stroke,
    pub closed: bool,
}

impl fmt::Debug for Polygon {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, r#"<path d="#)?;
        if self.points.len() > 0 {
            write!(f, "M {} {} ", self.points[0][0], self.points[0][1])?;
            for &p in &self.points[1..] {
                write!(f, "L {} {} ", p[0], p[1])?;
            }
            if self.closed {
                write!(f, "Z")?;
            }
        }
        write!(f, r#"" style="{:?};{:?}"/>"#, self.fill, self.stroke)
    }
}

pub fn polygon<T: Copy + Into<[f32; 2]>>(pts: &[T]) ->  Polygon {
    let mut points = Vec::with_capacity(pts.len());
    for p in pts {
        points.push((*p).into());
    }
    Polygon {
        points,
        fill: Fill::Color(black()),
        stroke: Stroke::None,
        closed: true,
    }
}

impl Polygon {
    pub fn open(mut self) -> Self {
        self.closed = false;
        self
    }

    pub fn fill<F>(mut self, fill: F) -> Self
    where F: Into<Fill> {
        self.fill = fill.into();
        self
    }

    pub fn stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = stroke;
        self
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct LineSegment {
    pub x1: f32,
    pub x2: f32,
    pub y1: f32,
    pub y2: f32,
    pub color: Color,
    pub width: f32,
}

impl fmt::Debug for LineSegment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
            r#"<path d="M {} {} L {} {}" style="stroke:{:?};stroke-width:{:?}"/>"#,
            self.x1, self.y1,
            self.x2, self.y2,
            self.color,
            self.width,
        )
    }
}

pub fn line_segment(x1: f32, y1: f32, x2: f32, y2: f32) -> LineSegment {
    LineSegment {
        x1, y1, x2, y2,
        color: black(),
        width: 1.0,
    }
}

impl LineSegment {
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn offset(mut self, dx: f32, dy: f32) -> Self {
        self.x1 += dx;
        self.y1 += dy;
        self.x2 += dx;
        self.y2 += dy;
        self
    }
}


#[derive(Clone, PartialEq)]
pub struct Text {
    pub x: f32, pub y: f32,
    pub text: String,
    pub color: Color,
    pub align: Align,
    pub size: f32,
}

impl fmt::Debug for Text {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
            r#"<text x="{}" y="{}" style="font-size:{}px;fill:{:?};{:?}"> {} </text>"#,
            self.x, self.y,
            self.size,
            self.color,
            self.align,
            self.text,
        )
    }
}

pub fn text<T: Into<String>>(x: f32, y: f32, txt: T) -> Text {
    Text {
        x, y,
        text: txt.into(),
        color: black(),
        align: Align::Left,
        size: 10.0,
    }
}

impl Text {
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn offset(mut self, dx: f32, dy: f32) -> Self {
        self.x += dx;
        self.y += dy;
        self
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum Align {
    Left, Right, Center
}

impl fmt::Debug for Align {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Align::Left => write!(f, "text-align:left"),
            Align::Right => write!(f, "text-align:right"),
            Align::Center => write!(f, "text-align:center"),
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct BeginSvg {
    pub w: f32,
    pub h: f32,
}

impl fmt::Debug for BeginSvg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}">"#,
            self.w,
            self.h,
        )
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct EndSvg;

impl fmt::Debug for EndSvg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "</svg>")
    }
}

pub struct Indentation {
    pub n: u32,
}

pub fn indent(n: u32) -> Indentation {
    Indentation { n }
}

impl Indentation {
    pub fn push(&mut self) {
        self.n += 1;
    }

    pub fn pop(&mut self) {
        self.n -= 1;
    }
}

impl fmt::Debug for Indentation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for _ in 0..self.n {
            write!(f, "    ")?;
        }
        Ok(())
    }
}

#[test]
fn foo() {
    println!("{:?}", BeginSvg { w: 800.0, h: 600.0 });
    println!("    {:?}",
        rectangle(20.0, 50.0, 200.0, 100.0)
            .fill(red())
            .stroke(Stroke::Color(black(), 3.0))
            .border_radius(5.0)
    );
    println!("    {:?}", text(25.0, 100.0, "Foo!").size(42.0).color(white()));
    println!("{:?}", EndSvg);
}
