use crate::{effects::globals::MinstdRand, utils::*};

pub trait PixelTransform {
    fn transform(&self, p: Vec2) -> Vec2;
}

#[derive(Clone)]
pub struct Rot2 {
    sin: f32,
    cos: f32,
}

impl Rot2 {
    pub fn from_angle(angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self { sin, cos }
    }

    pub fn transform(&self, p: Vec2) -> Vec2 {
        let x = p.x * self.cos - p.y * self.sin;
        let y = p.x * self.sin + p.y * self.cos;
        Vec2::new(x, y)
    }
}

#[derive(Clone)]
pub struct TurnScaleTransform {
    scale: f32,
    turn: f32,
    rot: Rot2,
}

impl TurnScaleTransform {
    pub fn from_scale_turn_raw(scale: f32, mut turn: f32, rand: &mut MinstdRand) -> Self {
        if rand.next_bool() {
            turn *= -1.;
        }

        turn *= 0.6;

        Self::from_scale_turn(scale, turn)
    }

    pub fn from_scale_turn(scale: f32, turn: f32) -> Self {
        Self {
            scale,
            turn,
            rot: Rot2::from_angle(turn),
        }
    }

    pub fn mode_2(rand: &mut MinstdRand) -> Self {
        let scale = 1.00 - 0.02 * rand.next_01_prom();
        let turn = 0.02 + 0.07 * rand.next_01_prom();
        Self::from_scale_turn_raw(scale, turn, rand)
    }

    pub fn mode_3(rand: &mut MinstdRand) -> Self {
        let scale = 0.85 + 0.10 * rand.next_01_prom();
        let turn = 0.01 + 0.015 * rand.next_01_prom();
        Self::from_scale_turn_raw(scale, turn, rand)
    }
}

impl PixelTransform for TurnScaleTransform {
    fn transform(&self, p: Vec2) -> Vec2 {
        self.rot.transform(p) * self.scale
    }
}

#[derive(Clone)]
pub struct DitherTurnScaleTransform {
    parts: [TurnScaleTransform; 2],
}

impl DitherTurnScaleTransform {
    pub fn from_scale_turn(parts: [TurnScaleTransform; 2]) -> Self {
        Self { parts }
    }

    pub fn from_scale_turn_raw(scale: [f32; 2], mut turn: [f32; 2], rand: &mut MinstdRand) -> Self {
        if rand.next_bool() {
            turn[0] *= -1.;
            turn[1] *= -1.;
        }

        turn[0] *= 0.6;
        turn[1] *= 0.6;

        Self::from_scale_turn([
            TurnScaleTransform::from_scale_turn(scale[0], turn[0]),
            TurnScaleTransform::from_scale_turn(scale[1], turn[1]),
        ])
    }

    pub fn mode_1(rand: &mut MinstdRand) -> Self {
        let scale1 = 0.985 - 0.12 * rand.next_01_prom().powi(2);
        let scale2 = scale1;

        let mut turn1 = 0.01 + 0.01 * rand.next_01_prom();
        let turn2 = turn1;

        if rand.next() % 3 == 1 {
            turn1 *= -1.;
        }

        Self::from_scale_turn_raw([scale1, scale2], [turn1, turn2], rand)
    }
}

impl PixelTransform for DitherTurnScaleTransform {
    fn transform(&self, p: Vec2) -> Vec2 {
        if (p.x as u32) % 2 == (p.y as u32) % 2 {
            self.parts[0].transform(p)
        } else {
            self.parts[1].transform(p)
        }
    }
}

#[derive(Clone)]
pub enum AnyTransform {
    TurnScale(TurnScaleTransform),
    DitherTurnScale(DitherTurnScaleTransform),
}

impl PixelTransform for AnyTransform {
    fn transform(&self, p: Vec2) -> Vec2 {
        match self {
            AnyTransform::TurnScale(mode) => mode.transform(p),
            AnyTransform::DitherTurnScale(mode) => mode.transform(p),
        }
    }
}

impl Into<AnyTransform> for TurnScaleTransform {
    fn into(self) -> AnyTransform {
        AnyTransform::TurnScale(self)
    }
}

impl Into<AnyTransform> for DitherTurnScaleTransform {
    fn into(self) -> AnyTransform {
        AnyTransform::DitherTurnScale(self)
    }
}
