use crate::{fx::Effect, painter::*, utils::*};
use core::f32;

pub struct Grid {
    pub y_roi: YRoi,
    pub x_inc: usize,
    pub y_inc: usize,
    pub fat_pixels: bool,
    pub val: u8,
}

impl Grid {
    pub fn new(s: &Settings, g: &Globals) -> Self {
        let inc = (s.fxw / 30) as usize;

        let fat_pixels = s.fxw >= 1800;

        let ph = g.frame as f32 * g.time_scale;
        let val = (65.
            + 45. * (ph * 0.06033).sin()
            + 35. * (ph * 0.04710 + 1.).sin()
            + 25. * (ph * 0.00523 - 1.).sin())
        .clamp(0., 255.) as u8;

        Self { y_roi: s.y_roi, x_inc: inc, y_inc: inc, fat_pixels, val }
    }
}

impl Effect for Grid {
    fn render(&self, img: &mut RgbaImage, _: &mut Minstd) {
        for y in (self.y_roi.min + 2..self.y_roi.max - 2).step_by(self.y_inc) {
            for x in (0..img.cols()).step_by(self.x_inc) {
                if self.fat_pixels {
                    saturate_rgb(&mut img[(y, x)], self.val);
                    saturate_rgb(&mut img[(y, x + 1)], self.val);
                    saturate_rgb(&mut img[(y + 1, x)], self.val);
                    saturate_rgb(&mut img[(y + 1, x + 1)], self.val);
                } else {
                    saturate_rgb(&mut img[(y, x)], self.val);
                }
            }
        }
    }
}

fn saturate_rgb(col: &mut Rgba, s: u8) {
    for ch in 0..3 {
        col[ch] = col[ch].max(s);
    }
}
