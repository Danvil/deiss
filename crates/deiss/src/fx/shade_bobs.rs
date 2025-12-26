use crate::{fx::Effect, utils::*};
use core::f32;

pub struct ShadeBobs {
    pub count: usize,
    pub center: Vec2i,
    pub floatframe: f32,
    pub micro_c: [[f32; 3]; 10],
    pub micro_f: [[f32; 4]; 10],
    pub micro_rad: [[f32; 4]; 10],
}

impl ShadeBobs {
    pub fn new(center: Vec2i, floatframe: f32, rand: &mut Minstd) -> Self {
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

            let mut a = self.center.x
                + (self.micro_rad[x][0] * (self.floatframe * self.micro_f[x][0]).cos()
                    + self.micro_rad[x][2] * (self.floatframe * self.micro_f[x][1]).cos())
                    as i32;

            let mut b = self.center.y
                + (self.micro_rad[x][1] * (self.floatframe * self.micro_f[x][2]).cos()
                    + self.micro_rad[x][3] * (self.floatframe * self.micro_f[x][3]).cos())
                    as i32;

            for _ in 0..4 {
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

                pxls[idx].sat_add_u3(delta[0]);
                pxls[idx + 1].sat_add_u3(delta[1]);
                pxls[idx - 1].sat_add_u3(delta[1]);
                pxls[idx + cols].sat_add_u3(delta[1]);
                pxls[idx - cols].sat_add_u3(delta[1]);
            }
        }
    }
}
