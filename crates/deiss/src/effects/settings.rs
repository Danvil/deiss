use crate::{effects::*, utils::*};

#[derive(Debug, Clone)]
pub struct Settings {
    pub volscale: f32,
    pub enable_map_dampening: bool,
    pub fxw: u32,
    pub fxh: u32,
    pub y_roi: YRoi,
    pub disp_bits: u32,
    pub gf: [f32; 6],
    pub mode_prefs: ModePrefs,
}

impl Settings {
    pub fn shape(&self) -> Shape2 {
        (self.fxh, self.fxw).into()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct YRoi {
    pub min: u32,
    pub max: u32,
}

impl YRoi {
    pub fn contains(&self, y: u32) -> bool {
        self.min <= y && y <= self.max
    }
}

#[derive(Debug, Clone)]
pub struct ModePrefs {
    // values range from 0 to 5 stars; default is 3
    values: Vec<u32>,
    total: u32,
}

impl ModePrefs {
    pub fn new() -> Self {
        Self { values: vec![0; 128], total: 0 }
    }
}

const NUM_MODES: u32 = 25;

impl ModePrefs {
    pub fn pick(&self, rng: &mut Minstd) -> ModeId {
        ModeId(1 + rng.next_idx(3))

        // if self.total == 0 {
        //     let mut m = 1 + rng.next_idx(NUM_MODES);
        //     if rng.next_idx(25) == 0 {
        //         m = 7;
        //     }
        //     if rng.next_idx(25) == 0 {
        //         m = 5;
        //     }
        //     m.into()
        // } else {
        //     todo!()
        // }
    }
}
