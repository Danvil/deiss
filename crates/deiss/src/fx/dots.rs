use crate::{fx::Effect, painter::*, utils::*};
use core::f32;

pub struct Dots {
    pub nodes: u32,
    pub center: Vec2i32,
    pub phase: f32,
    pub r: f32,
    pub rad: f32,
    pub col: [f32; 3],
}

impl Dots {
    pub fn new(spec: &FlowMapSpec, s: &Settings, g: &mut Globals) -> Option<Self> {
        if g.vol.current() <= g.avg_vol_narrow * 1.1 {
            return None;
        }

        let center = spec.center;

        let nodes = 3 + g.rand.next_idx(5);

        let phase = g.rand.next_idx(1000) as f32;

        let mut r =
            (if s.fxw == 320 { 2. } else { 3. } + 40. * (g.vol.current() / g.avg_vol_narrow - 1.1));
        r = r.max(1.);
        let r = if s.fxw == 320 { r.min(7.) } else { r.min(10.) };

        let rad = if s.fxw == 320 {
            (22 + g.rand.next_idx(6)) as f32
        } else {
            (34 + g.rand.next_idx(8)) as f32 * ((s.fxw as f32) / 1024.).max(1.)
        };

        let col = if s.disp_bits > 8 {
            let i = g.frame as f32 + g.chaser_offset;
            let f = 7. * (i * 0.007 + 29.).sin() + 5. * (i * 0.0057 + 27.).cos();
            [
                0.58 + 0.21 * (i * s.gf[0] + 20. - f).sin() + 0.21 * (i * s.gf[3] + 17. + f).cos(),
                0.58 + 0.21 * (i * s.gf[1] + 42. + f).sin() + 0.21 * (i * s.gf[4] + 26. - f).cos(),
                0.58 + 0.21 * (i * s.gf[2] + 57. - f).sin() + 0.21 * (i * s.gf[5] + 35. + f).cos(),
            ]
        } else {
            [1., 1., 1.]
        };

        Some(Dots { nodes, center, phase, r, rad, col })
    }
}

impl Effect for Dots {
    fn render(&self, img: &mut RgbaImage, _: &mut Minstd) {
        for n in 0..self.nodes {
            let (th_cos, th_sin) =
                ((n as f32) / (self.nodes as f32) * f32::consts::TAU + self.phase).sin_cos();
            let cx = self.center.x + (self.rad * th_cos) as i32;
            let cy = self.center.y + (self.rad * th_sin) as i32;

            for y in -10..=10 {
                for x in -10..=10 {
                    let scl = (self.r - ((x * x + y * y) as f32).sqrt()) * 25.;
                    if scl > 0. {
                        let pxl_idx = ((cy + y) as u32, (cx + x) as u32);
                        rgba_scl_add_inpl(&mut img[pxl_idx], scl, self.col);
                    }
                }
            }
        }
    }
}

fn rgba_scl_add_inpl(v: &mut Rgba, scl: f32, d: [f32; 3]) {
    for i in 0..3 {
        v[i] = v[i].saturating_add((scl * d[i]) as u8);
    }
}
