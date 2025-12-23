use crate::utils::*;

/// Pixel coordinate transformation mainly used for motion fields
pub trait PixelTransform {
    fn transform(&self, p: Vec2) -> Vec2;
}

/// Pixel transformation which rotates and then scales
#[derive(Debug, Clone)]
pub struct TurnScaleTransform {
    scale: f32,
    turn: f32,
    rot: Rot2,
}

impl TurnScaleTransform {
    pub fn from_scale_turn_raw(scale: f32, mut turn: f32, rand: &mut Minstd) -> Self {
        if rand.next_bool() {
            turn *= -1.;
        }

        turn *= 0.6;

        Self::from_scale_turn(scale, turn)
    }

    pub fn from_scale_turn(scale: f32, turn: f32) -> Self {
        Self { scale, turn, rot: Rot2::from_angle(turn) }
    }
}

impl PixelTransform for TurnScaleTransform {
    fn transform(&self, p: Vec2) -> Vec2 {
        self.rot.transform(p) * self.scale
    }
}

/// TODO
#[derive(Debug, Clone)]
pub struct DitherTurnScaleTransform {
    parts: [TurnScaleTransform; 2],
}

impl DitherTurnScaleTransform {
    pub fn from_scale_turn(parts: [TurnScaleTransform; 2]) -> Self {
        Self { parts }
    }

    pub fn from_scale_turn_raw(scale: [f32; 2], mut turn: [f32; 2], rand: &mut Minstd) -> Self {
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

/// Turn-Scale transformation with scale dependent on the pixel coordinate
#[derive(Debug, Clone)]
pub struct TurnVarScaleTf<S> {
    turn: f32,
    rot: Rot2,
    scale_f: S,
}

impl<S> TurnVarScaleTf<S> {
    pub fn new_raw(mut turn: f32, scale_f: S, rand: &mut Minstd) -> Self {
        if rand.next_bool() {
            turn *= -1.;
        }

        turn *= 0.6;

        Self { turn, rot: Rot2::from_angle(turn), scale_f }
    }
}

impl<S> PixelTransform for TurnVarScaleTf<S>
where
    S: ScaleF,
{
    fn transform(&self, p: Vec2) -> Vec2 {
        self.rot.transform(p) * self.scale_f.scale(p)
    }
}

/// Coordinate-dependent scale function used by [TurnVarScaleTf]
pub trait ScaleF {
    fn scale(&self, p: Vec2) -> f32;
}
