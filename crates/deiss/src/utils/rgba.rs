use crate::utils::{Image, Shape2};
use core::ops;

#[derive(Debug, Clone, Copy)]
pub struct Rgba(pub [u8; 4]);

impl Rgba {
    pub const BLACK: Self = Self([0, 0, 0, 255]);
    pub const WHITE: Self = Self([255, 255, 255, 255]);

    pub fn scaled(&self, scale: f32) -> Self {
        Self([
            (self.0[0] as f32 * scale) as u8,
            (self.0[1] as f32 * scale) as u8,
            (self.0[2] as f32 * scale) as u8,
            (self.0[3] as f32 * scale) as u8,
        ])
    }

    pub fn scale_f(&mut self, scale: f32) {
        for i in 0..3 {
            self[i] = (self[i] as f32 * scale) as u8;
        }
    }

    pub fn sat_add_u3(&mut self, delta: [u8; 3]) {
        for i in 0..3 {
            self[i] = self[i].saturating_add(delta[i]);
        }
    }

    pub fn sat_add_f_f3(&mut self, scale: f32, delta: [f32; 3]) {
        for i in 0..3 {
            self[i] = self[i].saturating_add((scale * delta[i]) as u8);
        }
    }
}

impl ops::Index<usize> for Rgba {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl ops::IndexMut<usize> for Rgba {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl ops::Div<u32> for Rgba {
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

pub type RgbaImage = Image<Rgba>;

impl RgbaImage {
    pub fn black(shape: Shape2) -> Self {
        Self::from_value(shape, Rgba::BLACK)
    }

    pub fn as_bytes(&self) -> &[u8] {
        let slice = self.as_slice();
        unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const u8, slice.len() * 4) }
    }
}
