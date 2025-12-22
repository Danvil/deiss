#[derive(Clone, Copy)]
pub struct Shape2(u32, u32);

impl Shape2 {
    /// Rows / height
    pub fn rows(&self) -> u32 {
        self.0
    }

    /// Columns / width
    pub fn cols(&self) -> u32 {
        self.1
    }

    /// Number of pixels covered by the shape
    pub fn len(&self) -> usize {
        (self.0 * self.1) as usize
    }

    pub fn offset(&self, (i, j): (u32, u32)) -> usize {
        (i * self.cols() + j) as usize
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
