use core::ops::Div;
use core::ops::{Index, IndexMut};

#[derive(Clone, Copy)]
pub struct Shape2(u32, u32);

impl Shape2 {
    pub fn rows(&self) -> u32 {
        self.0
    }

    pub fn cols(&self) -> u32 {
        self.1
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

#[derive(Clone, Copy)]
pub struct Rgba(pub [u8; 4]);

impl Rgba {
    pub const BLACK: Self = Self([0, 0, 0, 255]);
    pub const WHITE: Self = Self([255, 255, 255, 255]);
}

impl Div<u32> for Rgba {
    type Output = Self;

    fn div(self, rhs: u32) -> Self::Output {
        Self([
            self.0[0] / rhs as u8,
            self.0[1] / rhs as u8,
            self.0[2] / rhs as u8,
            self.0[3] / rhs as u8,
        ])
    }
}

#[derive(Clone)]
pub struct RgbaImage {
    shape: Shape2,
    buffer: Vec<Rgba>,
}

impl RgbaImage {
    pub fn black(shape: Shape2) -> Self {
        Self {
            shape,
            buffer: vec![Rgba::BLACK; (shape.0 * shape.1) as usize],
        }
    }

    pub fn shape(&self) -> Shape2 {
        self.shape
    }

    pub fn cols(&self) -> u32 {
        self.shape.cols()
    }

    pub fn rows(&self) -> u32 {
        self.shape.rows()
    }

    pub fn offset(&self, (i, j): (u32, u32)) -> usize {
        (i * self.shape.cols() + j) as usize
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self.buffer.as_ptr() as *const u8, self.buffer.len() * 4)
        }
    }

    pub fn apply(&mut self, mut f: impl FnMut(Rgba) -> Rgba) {
        for v in &mut self.buffer {
            *v = f(*v);
        }
    }
}

impl Index<(u32, u32)> for RgbaImage {
    type Output = Rgba;

    fn index(&self, coo: (u32, u32)) -> &Self::Output {
        let offset = self.offset(coo);
        &self.buffer[offset]
    }
}

impl IndexMut<(u32, u32)> for RgbaImage {
    fn index_mut(&mut self, coo: (u32, u32)) -> &mut Self::Output {
        let offset = self.offset(coo);
        &mut self.buffer[offset]
    }
}
