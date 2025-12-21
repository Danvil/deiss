#[derive(Clone, Copy)]
pub struct Shape2(u32, u32);

impl Shape2 {
    pub fn rows(&self) -> u32 {
        self.0
    }

    pub fn cols(&self) -> u32 {
        self.1
    }

    pub fn count(&self) -> u32 {
        self.0 * self.1
    }
}

impl From<(u32, u32)> for Shape2 {
    fn from((rows, cols): (u32, u32)) -> Self {
        Shape2(rows, cols)
    }
}

impl From<Shape2> for (u32, u32) {
    fn from(shape: Shape2) -> Self {
        (shape.rows(), shape.cols())
    }
}
