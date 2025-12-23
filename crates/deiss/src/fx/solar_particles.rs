use crate::{fx::Effect, painter::Globals, utils::*};
use core::f32;

pub struct SolarParticles {
    pub center: Vec2i32,
    pub count: usize,
}

impl SolarParticles {
    pub fn new(center: Vec2i32, solar_max: u32, g: &Globals) -> Self {
        let i = g.frame as f32;

        let solar_max = solar_max as f32;
        let fx = 3.
            + solar_max * 1.6
            + solar_max * 0.43 * (i * 0.05).sin()
            + solar_max * 0.43 * (i * 0.038 + 1.).sin();
        let count = (fx * 0.05) as usize;

        Self { center, count }
    }
}

impl Effect for SolarParticles {
    fn render(&self, img: &mut RgbaImage, rand: &mut Minstd) {
        let shape = img.shape();
        let cols = shape.cols() as usize;
        let pxls = img.as_slice_mut();

        for _ in 0..self.count {
            let (delta, r) = sample_disk(35, rand);
            let p = self.center + delta;
            let coo = (p.y as u32, p.x as u32);

            let idx = shape.offset(coo);

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

            pxls[idx].sat_add_u3(delta[0]);
            pxls[idx + 1].sat_add_u3(delta[1]);
            pxls[idx - 1].sat_add_u3(delta[1]);
            pxls[idx + cols].sat_add_u3(delta[1]);
            pxls[idx - cols].sat_add_u3(delta[1]);
            pxls[idx + cols + 1].sat_add_u3(delta[2]);
            pxls[idx + cols - 1].sat_add_u3(delta[2]);
            pxls[idx - cols + 1].sat_add_u3(delta[2]);
            pxls[idx - cols - 1].sat_add_u3(delta[2]);
        }
    }
}

fn sample_disk(r: u32, rand: &mut Minstd) -> (Vec2i32, u32) {
    let r2 = r * r;
    loop {
        let x = rand.next_idx(2 * r) as i32 - r as i32;
        let y = rand.next_idx(2 * r) as i32 - r as i32;
        let c2 = (x * x + y * y) as u32;
        if c2 < r2 {
            let c = (c2 as f32).sqrt() as u32;
            return (Vec2K::new(x, y), c);
        }
    }
}
