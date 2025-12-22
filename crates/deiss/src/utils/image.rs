use crate::utils::Shape2;
use core::ops;

#[derive(Clone)]
pub struct Image<T> {
    shape: Shape2,
    buffer: Vec<T>,
}

impl<T> Image<T> {
    pub fn from_vec(shape: Shape2, buffer: Vec<T>) -> Self {
        assert_eq!(shape.len(), buffer.len());
        Self { shape, buffer }
    }

    pub fn from_fn(shape: Shape2, mut f: impl FnMut((u32, u32)) -> T) -> Self {
        let mut buffer = Vec::with_capacity(shape.len());
        for i in 0..shape.rows() {
            for j in 0..shape.cols() {
                buffer.push(f((i, j)));
            }
        }
        Self { shape, buffer }
    }

    pub fn from_value(shape: Shape2, value: T) -> Self
    where
        T: Copy,
    {
        Self { shape, buffer: vec![value; shape.len()] }
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

    pub fn offset(&self, coo: (u32, u32)) -> usize {
        self.shape.offset(coo)
    }

    pub fn as_slice(&self) -> &[T] {
        &self.buffer
    }

    pub fn as_slice_mut(&mut self) -> &mut [T] {
        &mut self.buffer
    }

    pub fn apply(&mut self, mut f: impl FnMut(T) -> T)
    where
        T: Copy,
    {
        for v in &mut self.buffer {
            *v = f(*v);
        }
    }
}

impl<T> ops::Index<(u32, u32)> for Image<T> {
    type Output = T;

    fn index(&self, coo: (u32, u32)) -> &Self::Output {
        let offset = self.offset(coo);
        &self.buffer[offset]
    }
}

impl<T> ops::IndexMut<(u32, u32)> for Image<T> {
    fn index_mut(&mut self, coo: (u32, u32)) -> &mut Self::Output {
        let offset = self.offset(coo);
        &mut self.buffer[offset]
    }
}
