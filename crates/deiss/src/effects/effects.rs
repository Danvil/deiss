use crate::{effects::*, utils::*};
use core::{f32, ops};

#[derive(Debug, Clone)]
#[repr(C)]
pub enum EffectKind {
    Chasers = 0,
    Bar = 1,
    Dots = 2,
    Solar = 3,
    Grid = 4,
    Nuclide = 5,
    Shade = 6,
    Spectral = 7,
}

pub const NUM_EFFECTS: usize = 8;

#[derive(Debug, Clone)]
pub struct EffectFreq([u32; NUM_EFFECTS]);

impl EffectFreq {
    pub fn sample(&self, (min, max): (usize, usize), rand: &mut Minstd) -> Effects {
        let mut effect = Effects(self.0.map(|thresh| rand.next_idx(1000) < (thresh * 7) / 10));

        // clip num effects
        let mut n = effect.count();
        while n < min {
            for i in 0..NUM_EFFECTS {
                if !effect[i] && rand.next_idx(1000) < self.0[i] {
                    n += 1;
                    break;
                }
            }
        }
        for i in 0..NUM_EFFECTS {
            if self.0[i] >= 1000 {
                effect[i] = true;
            }
        }
        while n > max {
            let i = rand.next_idx(NUM_EFFECTS as u32) as usize;
            if effect[i] && self.0[i] < 1000 {
                effect[i] = false;
                n -= 1;
            }
        }

        if effect[EffectKind::Grid] {
            effect[EffectKind::Bar] = false;
        }

        effect
    }
}

impl Into<EffectFreq> for [u32; NUM_EFFECTS] {
    fn into(self) -> EffectFreq {
        EffectFreq(self)
    }
}

impl Default for EffectFreq {
    fn default() -> Self {
        Self([1000 / NUM_EFFECTS as u32; NUM_EFFECTS])
    }
}

impl ops::Index<EffectKind> for EffectFreq {
    type Output = u32;

    fn index(&self, index: EffectKind) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl ops::IndexMut<EffectKind> for EffectFreq {
    fn index_mut(&mut self, index: EffectKind) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

#[derive(Clone)]
pub struct Effects([bool; NUM_EFFECTS]);

impl Effects {
    pub fn count(&self) -> usize {
        self.0.iter().filter(|&&v| v).count()
    }
}

impl ops::Index<usize> for Effects {
    type Output = bool;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl ops::IndexMut<usize> for Effects {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl ops::Index<EffectKind> for Effects {
    type Output = bool;

    fn index(&self, index: EffectKind) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl ops::IndexMut<EffectKind> for Effects {
    fn index_mut(&mut self, index: EffectKind) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

pub trait Effect {
    fn render(&self, img: &mut RgbaImage, rand: &mut Minstd);
}

pub struct SolarParticles {
    pub center: (i32, i32),
    pub count: usize,
}

impl Effect for SolarParticles {
    fn render(&self, img: &mut RgbaImage, rand: &mut Minstd) {
        let shape = img.shape();
        let cols = shape.cols() as usize;
        let pxls = img.as_slice_mut();

        for _ in 0..self.count {
            let ((mut x, mut y), r) = sample_disk(35, rand);
            x += self.center.0;
            y += self.center.1;

            let idx = shape.offset((y as u32, x as u32));

            let col = pxls[idx];

            let mut delta = [[0; 3]; 3];

            for i in 0..3 {
                let i0 = (4 + (rand.next_idx(30) * (35 - r)) / 25) as u8;
                let i1 = i0 - 3;
                let i2 = i1 / 2;
                if col[i] < (207 - i0) as u8 {
                    delta[0][i] += i0;
                    delta[1][i] += i1;
                    delta[2][i] += i2;
                }
            }

            pxls[idx].sat_add_u8_3(delta[0]);
            pxls[idx + 1].sat_add_u8_3(delta[1]);
            pxls[idx - 1].sat_add_u8_3(delta[1]);
            pxls[idx + cols].sat_add_u8_3(delta[1]);
            pxls[idx - cols].sat_add_u8_3(delta[1]);
            pxls[idx + cols + 1].sat_add_u8_3(delta[2]);
            pxls[idx + cols - 1].sat_add_u8_3(delta[2]);
            pxls[idx - cols + 1].sat_add_u8_3(delta[2]);
            pxls[idx - cols - 1].sat_add_u8_3(delta[2]);
        }
    }
}

fn sample_disk(r: u32, rand: &mut Minstd) -> ((i32, i32), u32) {
    let r2 = r * r;
    loop {
        let x = rand.next_idx(2 * r) as i32 - r as i32;
        let y = rand.next_idx(2 * r) as i32 - r as i32;
        let c2 = (x * x + y * y) as u32;
        if c2 < r2 {
            let c = (c2 as f32).sqrt() as u32;
            return ((x, y), c);
        }
    }
}

pub struct Dots {
    pub nodes: u32,
    pub center: Vec2,
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
            let i = (g.frame + s.chaser_offset as u64) as f32;
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
    fn render(&self, img: &mut RgbaImage, rand: &mut Minstd) {
        for n in 0..self.nodes {
            let (th_cos, th_sin) =
                ((n as f32) / (self.nodes as f32) * f32::consts::TAU + self.phase).sin_cos();
            let cx = (self.center.x + self.rad * th_cos) as i32;
            let cy = (self.center.y + self.rad * th_sin) as i32;

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

pub struct ShadeBobs {
    pub count: usize,
    pub center: Vec2,
    pub floatframe: f32,
    pub micro_c: [[f32; 3]; 10],
    pub micro_f: [[f32; 4]; 10],
    pub micro_rad: [[f32; 4]; 10],
}

impl ShadeBobs {
    pub fn new(center: Vec2, floatframe: f32, rand: &mut Minstd) -> Self {
        let mut micro_c = [[0.; 3]; 10];
        let mut micro_f = [[0.; 4]; 10];
        let mut micro_rad = [[0.; 4]; 10];

        for i in 0..10 {
            for j in 0..3 {
                micro_c[i][j] = 0.08 + 0.09 * rand.next_01_prom();
            }
            for j in 0..4 {
                micro_f[i][j] = 0.1 + 0.05 * rand.next_01_prom();
            }
            for j in 0..4 {
                micro_rad[i][j] = 2.0 + 2.8 * rand.next_01_prom();
            }
        }

        Self { count: 10, center, floatframe, micro_c, micro_f, micro_rad }
    }
}

impl Effect for ShadeBobs {
    fn render(&self, img: &mut RgbaImage, rand: &mut Minstd) {
        let shape = img.shape();
        let cols = shape.cols() as usize;
        let pxls = img.as_slice_mut();

        for x in 0..self.count {
            let col: [u32; 3] = core::array::from_fn(|c| {
                (1. + (self.floatframe * self.micro_c[x][c]).sin()) as u32
            });

            let mut a = (self.center.x
                + self.micro_rad[x][0] * (self.floatframe * self.micro_f[x][0]).cos()
                + self.micro_rad[x][2] * (self.floatframe * self.micro_f[x][1]).cos())
                as i32;

            let mut b = (self.center.y
                + self.micro_rad[x][1] * (self.floatframe * self.micro_f[x][2]).cos()
                + self.micro_rad[x][3] * (self.floatframe * self.micro_f[x][3]).cos())
                as i32;

            for k in 0..4 {
                a += rand.next_idx(5) as i32 - 2;
                b += rand.next_idx(5) as i32 - 2;

                let idx = shape.offset((b as u32, a as u32));

                let mut delta = [[0; 3]; 2];

                for c in 0..3 {
                    if col[c] == 1 {
                        delta[0][c] += 5;
                        delta[1][c] += 3;
                    }
                }

                pxls[idx].sat_add_u8_3(delta[0]);
                pxls[idx + 1].sat_add_u8_3(delta[1]);
                pxls[idx - 1].sat_add_u8_3(delta[1]);
                pxls[idx + cols].sat_add_u8_3(delta[1]);
                pxls[idx - cols].sat_add_u8_3(delta[1]);
            }
        }
    }
}
