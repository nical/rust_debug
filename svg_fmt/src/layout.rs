use crate::svg::{Rectangle, rectangle};

#[derive(Copy, Clone, Debug)]
pub struct VerticalLayout {
    pub start: [f32; 2],
    pub y: f32,
    pub width: f32,
}

impl VerticalLayout {
    pub fn new(x: f32, y: f32, width: f32) -> Self {
        VerticalLayout {
            start: [x, y],
            y,
            width,
        }
    }

    pub fn advance(&mut self, by: f32) {
        self.y += by;
    }

    pub fn push_rectangle(&mut self, height: f32) -> Rectangle {
        let rect = rectangle(self.start[0], self.y, self.width, height);

        self.y += height;

        rect
    }

    pub fn total_rectangle(&self) -> Rectangle {
        rectangle(
            self.start[0], self.start[1],
            self.width, self.y,
        )
    }

    pub fn start_here(&mut self) {
        self.start[1] = self.y;
    }
}
