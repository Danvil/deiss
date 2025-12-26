use crate::{fx::Effect, painter::*, utils::*};
use core::f32;

pub struct DiminishCenter {
    pub y_roi: YRoi,
    pub center: Vec2i,
    pub center_mode: bool,
    pub center_dwindle: f32,
}

impl DiminishCenter {
    pub fn new(center: Vec2i, center_mode: bool, center_dwindle: f32, s: &Settings) -> Self {
        Self { y_roi: s.y_roi, center, center_mode, center_dwindle }
    }
}
impl Effect for DiminishCenter {
    fn render(&self, img: &mut RgbaImage, _: &mut Minstd) {
        let cj = self.center.x as u32;
        let ci = self.center.y as u32;

        if self.center_mode {
            // center cross
            img[(ci, ci)].scale_f(self.center_dwindle);
            img[(ci, ci - 1)].scale_f(self.center_dwindle);
            img[(ci, ci + 1)].scale_f(self.center_dwindle);
            img[(ci - 1, ci)].scale_f(self.center_dwindle);
            img[(ci + 1, ci)].scale_f(self.center_dwindle);
        } else {
            // vertial line
            for i in self.y_roi.min..self.y_roi.max {
                img[(i, cj)].scale_f(self.center_dwindle);
                img[(i, cj - 1)].scale_f(self.center_dwindle);
                img[(i, cj + 1)].scale_f(self.center_dwindle);
            }
        }
    }
}
