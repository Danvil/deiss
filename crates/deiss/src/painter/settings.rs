use crate::{painter::*, utils::*};

#[derive(Debug, Clone)]
pub struct Settings {
    pub volscale: f32,
    pub enable_map_dampening: bool,
    pub fxw: u32,
    pub fxh: u32,
    pub y_roi: YRoi,
    pub gf: [f32; 6],
    pub mode_prefs: ModePrefs,
}

impl Settings {
    pub fn shape(&self) -> Shape2 {
        (self.fxh, self.fxw).into()
    }
}

pub fn generate_gf(rand: &mut Minstd) -> [f32; 6] {
    core::array::from_fn(|_| ((rand.next_idx(1000) as f32) * 0.001) * 0.01 + 0.02)
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
    /// If set always pick this mode
    priority: Option<ModeId>,

    /// Weights to select modes (also indicates valid modes). Weights are 0 to 5 stars.
    weights: Vec<(ModeId, u32)>,
}

impl ModePrefs {
    pub fn new(modes: &[u32]) -> Self {
        Self { priority: None, weights: modes.into_iter().map(|&i| (ModeId(i), 3)).collect() }
    }
}

impl ModePrefs {
    pub fn pick(&self, rng: &mut Minstd) -> ModeId {
        if let Some(m) = self.priority {
            m
        } else {
            // weighted sampling based on preferences
            let total = self.weights.iter().map(|(_, w)| w).sum::<u32>();
            if total == 0 {
                return ModeId(1);
            }

            let mut rnd = rng.next_idx(total);
            for &(m, w) in self.weights.iter() {
                if rnd <= w {
                    return m;
                }
                rnd -= w;
            }
            unreachable!()
        }
    }

    pub fn weights(&self) -> &[(ModeId, u32)] {
        &self.weights
    }

    pub fn weights_mut(&mut self) -> &mut [(ModeId, u32)] {
        &mut self.weights
    }

    pub fn priority(&self) -> Option<ModeId> {
        self.priority
    }

    pub fn set_priority(&mut self, priority: Option<ModeId>) {
        self.priority = priority;
    }
}
