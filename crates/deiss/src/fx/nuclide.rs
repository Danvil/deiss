use crate::{fx::Effect, painter::*, utils::*};
use core::f32;

pub struct Nuclide {
    pub nodes: u32,
    pub center: Vec2i,
    pub phase: f32,
    pub r: f32,
    pub rad: f32,
    pub col: [f32; 3],
}

impl Nuclide {
    pub fn new_nuclide(center: Vec2i, s: &Settings, g: &mut Globals) -> Self {
        // skip 11/12 runs
        let nodes = if g.rand.next_idx(12) == 0 { 3 + g.rand.next_idx(5) } else { 0 };

        let phase = g.rand.next_idx(1000) as f32;

        let r = (3 + g.rand.next_idx(8)) as f32;

        let rad = (34 + g.rand.next_idx(8)) as f32;

        let t = g.frame as f32 + g.chaser_offset;
        let f = 7. * (t * 0.007 + 29.).sin() + 5. * (t * 0.0057 + 27.).cos();
        let dat = color_gen(s.gf, f, t, [0.25, 0.25], [20., 17., 42., 26., 57., 35.]);
        let col = [0.50 + dat[0] + dat[1], 0.5 + dat[2] + dat[3], 0.5 + dat[4] + dat[5]];
        // let col = [
        //     0.5 + 0.25 * (t * s.gf[0] + 20. - f).sin() + 0.25 * (t * s.gf[3] + 17. + f).cos(),
        //     0.5 + 0.25 * (t * s.gf[1] + 42. + f).sin() + 0.25 * (t * s.gf[4] + 26. - f).cos(),
        //     0.5 + 0.25 * (t * s.gf[2] + 57. - f).sin() + 0.25 * (t * s.gf[5] + 35. + f).cos(),
        // ];

        Self { center, nodes, rad, r, phase, col }
    }

    pub fn new_beat_dots(center: Vec2i, s: &Settings, g: &mut Globals) -> Self {
        // skip if volume too low
        let nodes =
            if g.vol.current() <= g.avg_vol_narrow * 1.1 { 0 } else { 3 + g.rand.next_idx(5) };

        let phase = g.rand.next_idx(1000) as f32;

        let r = (3. + 40. * (g.vol.current() / g.avg_vol_narrow - 1.1)).clamp(1., 10.);

        let rad = (34 + g.rand.next_idx(8)) as f32 * ((s.fxw as f32) / 1024.).max(1.);

        let t = g.frame as f32 + g.chaser_offset;
        let f = 7. * (t * 0.007 + 29.).sin() + 5. * (t * 0.0057 + 27.).cos();
        let dat = color_gen(s.gf, f, t, [0.21, 0.21], [20., 17., 42., 26., 57., 35.]);
        let col = [0.58 + dat[0] + dat[1], 0.5 + dat[2] + dat[3], 0.5 + dat[4] + dat[5]];
        // let col = [
        //     0.58 + 0.21 * (i * s.gf[0] + 20. - f).sin() + 0.21 * (i * s.gf[3] + 17. + f).cos(),
        //     0.58 + 0.21 * (i * s.gf[1] + 42. + f).sin() + 0.21 * (i * s.gf[4] + 26. - f).cos(),
        //     0.58 + 0.21 * (i * s.gf[2] + 57. - f).sin() + 0.21 * (i * s.gf[5] + 35. + f).cos(),
        // ];

        Self { nodes, center, phase, r, rad, col }
    }
}

impl Effect for Nuclide {
    fn render(&self, img: &mut RgbaImage, _: &mut Minstd) {
        const RAD: i32 = 10;

        for n in 0..self.nodes {
            let (th_cos, th_sin) =
                ((n as f32) / (self.nodes as f32) * f32::consts::TAU + self.phase).sin_cos();
            let p = self.center + (Vec2f::new(th_cos, th_sin) * self.rad).cast();

            for x in -RAD..RAD {
                for y in -RAD..RAD {
                    let val = (self.r - ((x * x + y * y) as f32).sqrt()) * 25.;
                    if val > 0. {
                        let coo = ((p.y + y) as u32, (p.x + x) as u32);
                        img[coo].sat_add_f_f3(val, self.col);
                    }
                }
            }
        }
    }
}
