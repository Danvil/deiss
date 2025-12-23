use crate::{fx::Effect, painter::*, utils::*};
use core::f32;

pub struct SnackBar {
    pub y_roi: YRoi,
    pub center: Vec2i32,
    pub frame: [f32; 3],
}

impl SnackBar {
    pub fn new(center: Vec2i32, s: &Settings, g: &Globals) -> Self {
        let speed_mult = 0.6;
        let chromatic_dispersion = 4.;

        let frame = g.floatframe + g.chaser_offset * speed_mult;

        let dcg = 3.5
            * chromatic_dispersion
            * ((g.floatframe * 0.03 + 1.).sin() + (g.floatframe * 0.04 + 3.).cos());

        let dcb = -3.5
            * chromatic_dispersion
            * ((g.floatframe * 0.05 + 2.).cos() + (g.floatframe * 0.06 + 4.).sin());

        Self { y_roi: s.y_roi, center, frame: [frame, frame + dcg, frame + dcb] }
    }
}

impl Effect for SnackBar {
    fn render(&self, img: &mut RgbaImage, _: &mut Minstd) {
        let s = img.cols() as f32 / 640.;

        for ch in 0..3 {
            let t = self.frame[ch] * 0.55 / (0.08 * 20.);

            let d1 = Vec2::new(
                16. * (t * 0.1102 + 10.).cos() + 15. * (t * 0.1312 + 20.).cos(),
                15. * (t * 0.1204 + 40.).cos() + 10. * (t * 0.1715 + 30.).cos(),
            );

            let d2 = Vec2::new(
                14. * (t * 0.1213 + 33.).cos() + 13. * (t * 0.1408 + 15.).cos(),
                13. * (t * 0.1304 + 12.).cos() + 11. * (t * 0.1103 + 21.).cos(),
            );

            let n = (s * 50.) as usize;
            let n_inv = 1.0 / (n as f32);

            for k in 0..n {
                let q = (k as f32) * n_inv;
                let d = d1 * q + d2 * (1.0 - q);
                let p = self.center + (d * s).cast();
                let coo = (p.y as u32, p.x as u32);
                if self.y_roi.contains(coo.0) {
                    let v = &mut img[coo][ch];
                    if *v < 223 {
                        *v += 16;
                    }
                }
            }
        }
    }
}
