use crate::{fx::Effect, painter::*, utils::*};
use core::f32;

pub struct TwoChasers {
    pub y_roi: YRoi,
    pub center: Vec2i,
    pub passes: usize,
    pub frame: f32,
    pub time_scale: f32,
}

impl TwoChasers {
    pub fn new(center: Vec2i, passes: usize, s: &Settings, g: &Globals) -> Self {
        Self {
            y_roi: s.y_roi,
            center,
            passes,
            frame: (g.floatframe + g.chaser_offset) * g.time_scale,
            time_scale: g.time_scale,
        }
    }
}

impl Effect for TwoChasers {
    fn render(&self, img: &mut RgbaImage, _: &mut Minstd) {
        let s = img.cols() as f32 / 640.;
        let n = (20. * s) as usize;

        let mut t = self.frame;
        for _ in 0..n {
            t += 0.08 * self.time_scale * 20. / n as f32;

            for pass in 0..self.passes {
                let delta = if pass == 0 {
                    Vec2f::new(
                        74. * (t * 0.1102 + 10.).cos() + 65. * (t * 0.1312 + 20.).cos(),
                        54. * (t * 0.1204 + 40.).cos() + 55. * (t * 0.1715 + 30.).cos(),
                    )
                } else {
                    Vec2f::new(
                        64. * (t * 0.1213 + 33.).cos() + 55. * (t * 0.1408 + 15.).cos(),
                        52. * (t * 0.1304 + 12.).cos() + 51. * (t * 0.1103 + 21.).cos(),
                    )
                };
                let p = self.center + (delta * s).cast();

                let coo = (p.y as u32, p.x as u32);

                if self.y_roi.contains(coo.0) {
                    let col = &mut img[coo];

                    *col = Rgba([
                        255 - ((255 - col[0]) as f32 * 0.6) as u8,
                        255 - ((255 - col[1]) as f32 * 0.6) as u8,
                        255 - ((255 - col[2]) as f32 * 0.6) as u8,
                        255,
                    ]);
                }
            }
        }
    }
}
