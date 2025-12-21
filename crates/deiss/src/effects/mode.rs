use crate::{effects::globals::MinstdRand, utils::*};

pub trait ModeKernel {
    fn transform(&self, p: Vec2) -> Vec2;
}

#[derive(Clone)]
pub struct TurnScaleMode {
    scale: f32,
    turn: f32,
    sin_turn: f32,
    cos_turn: f32,
}

impl ModeKernel for TurnScaleMode {
    fn transform(&self, p: Vec2) -> Vec2 {
        let x = p.x * self.cos_turn - p.y * self.sin_turn;
        let y = p.x * self.sin_turn + p.y * self.cos_turn;
        Vec2::new(x, y) * self.scale
    }
}

impl TurnScaleMode {
    pub fn from_scale_turn_raw(scale: f32, mut turn: f32, rand: &mut MinstdRand) -> Self {
        if rand.next_bool() {
            turn *= -1.;
        }

        turn *= 0.6;

        Self::from_scale_turn(scale, turn)
    }

    pub fn from_scale_turn(scale: f32, turn: f32) -> Self {
        let (sin_turn, cos_turn) = turn.sin_cos();
        Self {
            scale,
            turn,
            sin_turn,
            cos_turn,
        }
    }

    pub fn mode_1(rand: &mut MinstdRand) -> Self {
        let scale = 0.985 - 0.12 * rand.next_01_prom().powi(2);
        let mut turn = 0.01 + 0.01 * rand.next_01_prom();
        if scale > 0.97 && rand.next() % 3 == 1 {
            turn *= -1.;
        }
        Self::from_scale_turn_raw(scale, turn, rand)
    }
}
