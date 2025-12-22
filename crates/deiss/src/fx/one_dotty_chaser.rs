use crate::{fx::Effect, painter::*, utils::*};
use core::f32;
use std::sync::{Arc, Mutex};

pub struct OneDottyChaser {
    pub y_roi: YRoi,
    pub center: Vec2i32,
    pub time: f32,
    pub chasers: Arc<Mutex<Chasers>>,
}

impl OneDottyChaser {
    pub fn new(center: Vec2i32, s: &Settings, g: &Globals) -> Self {
        let time_scale = if 10. <= g.fps_at_last_mode_switch && g.fps_at_last_mode_switch < 120. {
            30. / g.fps_at_last_mode_switch
        } else {
            1.
        };

        Self { y_roi: s.y_roi, center, time: g.floatframe * time_scale, chasers: g.chasers.clone() }
    }
}

impl Effect for OneDottyChaser {
    fn render(&self, img: &mut RgbaImage, _: &mut Minstd) {
        let s = img.cols() as f32 / 640.;
        let t = self.time;

        let delta = Vec2::new(
            64. * (t * 0.0613 + 33.).cos() + 55. * (t * 0.0708 + 15.).cos(),
            52. * (t * 0.0704 + 12.).cos() + 51. * (t * 0.0503 + 21.).cos(),
        );
        let p = self.center + (delta * s).cast();
        let coo = (p.y as u32, p.x as u32);

        if self.y_roi.contains(coo.0) {
            self.chasers.lock().unwrap().push(Chaser {
                coo,
                color: Rgba([
                    (127. + 126. * (t * 0.0613 + 33.).sin()) as u8,
                    (127. + 126. * (t * 0.0713 + 33.).sin()) as u8,
                    (127. + 126. * (t * 0.0513 + 33.).sin()) as u8,
                    255,
                ]),
            });

            for ch in &mut self.chasers.lock().unwrap().items {
                img[ch.coo] = ch.color;

                // TODO FXW >= 880
                img[(ch.coo.0, ch.coo.1 + 1)] = ch.color;
                img[(ch.coo.0 + 1, ch.coo.1)] = ch.color;
                img[(ch.coo.0 + 1, ch.coo.1 + 1)] = ch.color;

                ch.coo.1 += 1;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Chasers {
    items: Vec<Chaser>,
    idx: usize,
}

impl Chasers {
    pub fn push(&mut self, chaser: Chaser) {
        self.idx = (self.idx + 1) % self.items.len();
        self.items[self.idx] = chaser;
    }
}

impl Default for Chasers {
    fn default() -> Self {
        Self {
            items: (0..20).map(|_| Chaser { coo: (0, 0), color: Rgba::BLACK }).collect(),
            idx: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Chaser {
    coo: (u32, u32),
    color: Rgba,
}
