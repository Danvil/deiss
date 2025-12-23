use crate::{painter::*, utils::Minstd};
use core::ops;

pub struct ModeBlueprint {
    pub effect_freq: EffectFreq,
    pub solar_max: u32,
    pub center_dwindle: f32,
    pub effect_count: [u32; 2],
    pub motion_dampened: bool,
    pub tf_gen: Box<dyn GenerateTransform>,
}

impl ModeBlueprint {
    pub fn generate_transform(&self, rand: &mut Minstd) -> AnyTransform {
        self.tf_gen.generate_transform(rand)
    }
}

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

#[derive(Debug, Clone)]
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
