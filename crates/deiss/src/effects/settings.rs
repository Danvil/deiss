use crate::{effects::globals::MinstdRand, utils::Shape2};
use std::ops::Index;

#[derive(Debug, Clone)]
pub struct Settings {
    pub volscale: f32,
    pub enable_map_dampening: bool,
    pub fxw: u32,
    pub fxh: u32,
    pub fx_ycut: u32,
    pub disp_bits: u32,
    pub chaser_offset: u32,
    pub gf: [f32; 6],
    pub mode_prefs: ModePrefs,
    pub mode_settings: ModeSettingsVec,
}

impl Settings {
    pub fn shape(&self) -> Shape2 {
        (self.fxh, self.fxw).into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Mode(pub u32);

// impl Mode {
//     pub fn new(m: u32) -> Self {
//         assert!(m > 0); // TODO is this needed?
//         Self(m)
//     }
// }

impl From<u32> for Mode {
    fn from(value: u32) -> Self {
        Mode(value)
    }
}

impl From<Mode> for u32 {
    fn from(mode: Mode) -> Self {
        mode.0
    }
}

#[derive(Debug, Clone)]
pub struct ModeSettings {
    pub motion_dampened: bool,
    pub rotation_dither: bool,
    pub custom_motion_vector: bool,
}

#[derive(Debug, Clone)]
pub struct ModeSettingsVec(Vec<ModeSettings>);

impl ModeSettingsVec {
    pub fn from_vec(items: Vec<ModeSettings>) -> Self {
        ModeSettingsVec(items)
    }
}

impl Index<Mode> for ModeSettingsVec {
    type Output = ModeSettings;

    fn index(&self, index: Mode) -> &Self::Output {
        &self.0[index.0 as usize]
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
        Self {
            values: vec![0; 128],
            total: 0,
        }
    }
}

const NUM_MODES: u32 = 25;

impl ModePrefs {
    pub fn pick(&self, rng: &mut MinstdRand) -> Mode {
        Mode(1 + rng.next_idx(3))

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
