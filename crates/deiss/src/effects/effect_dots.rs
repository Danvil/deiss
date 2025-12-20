use std::cell::RefCell;

use crate::{
    audio::AudioSamples,
    effects::{Effect, Rgba, RgbaImage},
};

pub struct EffectDots {
    rng: RefCell<u32>,
}

impl Default for EffectDots {
    fn default() -> Self {
        Self {
            rng: RefCell::new(1),
        }
    }
}

impl EffectDots {
    pub fn rand(&self) -> u32 {
        let mut x = self.rng.borrow_mut();
        *x = ((*x as u64 * 48_271) % 2_147_483_647) as u32;
        *x
    }
}

impl Effect for EffectDots {
    fn render(&self, samples: &AudioSamples, img: &mut RgbaImage) {
        // for i in 0..img.rows() {
        //     for j in 0..img.cols() {
        //         let r = (i % 256) as u8;
        //         let g = (j % 256) as u8;
        //         let b = ((i * j) % 256) as u8;

        //         img[(i, j)] = Rgba([r, g, b, 255]);
        //     }
        // }

        for k in 0..2000 {
            let i = self.rand() % img.shape().rows();
            let j = self.rand() % img.shape().cols();

            let r = 192 + (self.rand() % 64) as u8;
            let g = 192 + (self.rand() % 64) as u8;
            let b = 192 + (self.rand() % 64) as u8;

            img[(i, j)] = Rgba([r, g, b, 255]);
        }
    }
}
