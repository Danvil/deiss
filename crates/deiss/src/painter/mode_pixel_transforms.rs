use crate::{
    painter::{
        DitherTurnScaleTransform, PixelTransform, ScaleF, TurnScaleTransform, TurnVarScaleTf,
    },
    utils::*,
};

const RMULT: f32 = 1.0; // 640/FXW
const PROTECTIVE_FACTOR: f32 = 1.0; // min(640/FWX, 1)

// === Mode 1

pub type Mode1Tf = DitherTurnScaleTransform;

pub fn mode_1_tf(rand: &mut Minstd) -> DitherTurnScaleTransform {
    let scale1 = 0.985 - 0.12 * rand.next_01_prom().powi(2);
    let scale2 = scale1;

    let mut turn1 = 0.01 + 0.01 * rand.next_01_prom();
    let turn2 = turn1;

    if rand.next() % 3 == 1 {
        turn1 *= -1.;
    }

    Mode1Tf::from_scale_turn_raw([scale1, scale2], [turn1, turn2], rand)
}

// === Mode 2

pub type Mode2Tf = TurnScaleTransform;

pub fn mode_2_tf(rand: &mut Minstd) -> Mode2Tf {
    let scale = 1.00 - 0.02 * rand.next_01_prom();
    let turn = 0.02 + 0.07 * rand.next_01_prom();
    Mode2Tf::from_scale_turn_raw(scale, turn, rand)
}

// === Mode 3

pub type Mode3Tf = TurnVarScaleTf<Mode3Scale>;

pub fn mode_3_tf(rand: &mut Minstd) -> Mode3Tf {
    let scale = 0.85 + 0.10 * rand.next_01_prom();
    let turn = 0.01 + 0.015 * rand.next_01_prom();
    Mode3Tf::new_raw(turn, Mode3Scale(scale), rand)
}

#[derive(Debug, Clone)]
pub struct Mode3Scale(f32);

impl ScaleF for Mode3Scale {
    fn scale(&self, _: Vec2) -> f32 {
        self.0
    }
}

// === Mode 4

pub type Mode4Tf = TurnVarScaleTf<Mode4Scale>;

pub fn mode_4_tf(rand: &mut Minstd) -> Mode4Tf {
    let turn = 0.007 + 0.02 * rand.next_01_prom();
    Mode4Tf::new_raw(turn, Mode4Scale, rand)
}

#[derive(Debug, Clone)]
pub struct Mode4Scale;

impl ScaleF for Mode4Scale {
    fn scale(&self, p: Vec2) -> f32 {
        let r = p.norm() * RMULT;
        0.9 + r * 0.0025 * 0.14
    }
}

// === Mode 5

pub type Mode5Tf = TurnVarScaleTf<Mode5Scale>;

pub fn mode_5_tf(has_nuclide_effect: bool, rand: &mut Minstd) -> Mode5Tf {
    let turn = 0.01 + 0.03 * rand.next_01_prom();
    let f1 = 0.05 + 0.05 * rand.next_01_prom() + 0.07 * rand.next_01_prom();
    let f2 = 0.99 - 0.01 * rand.next_01_prom() - 0.02 * rand.next_01_prom();
    Mode5Tf::new_raw(turn, Mode5Scale { has_nuclide_effect, f1, f2 }, rand)
}

#[derive(Debug, Clone)]
pub struct Mode5Scale {
    has_nuclide_effect: bool,
    f1: f32,
    f2: f32,
}

impl ScaleF for Mode5Scale {
    fn scale(&self, p: Vec2) -> f32 {
        let mut r = p.norm() * RMULT / 200.;

        if self.has_nuclide_effect {
            r = r.sqrt();
        } else {
            r *= 1.7;
        }

        (self.f2 - self.f1 * r - 1.) * PROTECTIVE_FACTOR + 1.
    }
}

// === Mode 7

pub type Mode7Tf = TurnVarScaleTf<Mode7Scale>;

pub fn mode_7_tf(rand: &mut Minstd) -> Mode7Tf {
    let turn = 0.01 + 0.01 * rand.next_01_prom();
    let f1 = 0.92 + 0.01 * rand.next_01_prom();
    let f2 = 0.0006 + 0.0005 * rand.next_01_prom();
    let rand_array = (0..2345).map(|_| rand.next_idx(100) as f32 * 0.0005).collect();
    Mode7Tf::new_raw(turn, Mode7Scale { f1, f2, rand_array }, rand)
}

#[derive(Debug, Clone)]
pub struct Mode7Scale {
    f1: f32,
    f2: f32,
    rand_array: Vec<f32>,
}

impl ScaleF for Mode7Scale {
    fn scale(&self, p: Vec2) -> f32 {
        let r = p.norm() * RMULT * self.f2;
        let scale = (self.f1 - r - 1.) * PROTECTIVE_FACTOR + 1.;
        let idx = (p.x + 1000.) as usize + (p.y + 1000.) as usize * 2000;
        scale + self.rand_array[idx % self.rand_array.len()]
    }
}

// === Mode 8

pub type Mode8Tf = TurnVarScaleTf<Mode8Scale>;

pub fn mode_8_tf(rand: &mut Minstd) -> Mode8Tf {
    let turn = 0.05 * rand.next_01_prom();
    let f1 = rand.next_01_prom().powi(3) * 8. + 1.5;
    Mode8Tf::new_raw(turn, Mode8Scale { f1 }, rand)
}

#[derive(Debug, Clone)]
pub struct Mode8Scale {
    f1: f32,
}

impl ScaleF for Mode8Scale {
    fn scale(&self, p: Vec2) -> f32 {
        let r = p.norm() * RMULT;
        0.85 + 0.1 * (self.f1 * r.sqrt()).sin()
    }
}

// === Mode 9

pub type Mode9Tf = TurnVarScaleTf<Mode9Scale>;

pub fn mode_9_tf(rand: &mut Minstd) -> Mode9Tf {
    let turn = 0.01 + 0.03 * rand.next_01_prom();
    let f1 = 0.98 + 0.01 * rand.next_01_prom();
    let f2 = 0.0009 + 0.0012 * rand.next_01_prom();
    Mode9Tf::new_raw(turn, Mode9Scale { f1, f2 }, rand)
}

#[derive(Debug, Clone)]
pub struct Mode9Scale {
    f1: f32,
    f2: f32,
}

impl ScaleF for Mode9Scale {
    fn scale(&self, p: Vec2) -> f32 {
        let r = p.norm() * self.f2 * RMULT;
        (self.f1 - r - 1.) * PROTECTIVE_FACTOR + 1.
    }
}

// === Mode Tf enum

#[derive(Debug, Clone)]
pub enum AnyTransform {
    Mode1(Mode1Tf),
    Mode2(Mode2Tf),
    Mode3(Mode3Tf),
    Mode4(Mode4Tf),
    Mode5(Mode5Tf),
    Mode7(Mode7Tf),
    Mode8(Mode8Tf),
    Mode9(Mode9Tf),
}

impl PixelTransform for AnyTransform {
    fn transform(&self, p: Vec2) -> Vec2 {
        match self {
            AnyTransform::Mode1(tf) => tf.transform(p),
            AnyTransform::Mode2(tf) => tf.transform(p),
            AnyTransform::Mode3(tf) => tf.transform(p),
            AnyTransform::Mode4(tf) => tf.transform(p),
            AnyTransform::Mode5(tf) => tf.transform(p),
            AnyTransform::Mode7(tf) => tf.transform(p),
            AnyTransform::Mode8(tf) => tf.transform(p),
            AnyTransform::Mode9(tf) => tf.transform(p),
        }
    }
}

macro_rules! impl_into_any_transform {
    ($transform_type:ty, $variant:ident) => {
        impl Into<AnyTransform> for $transform_type {
            fn into(self) -> AnyTransform {
                AnyTransform::$variant(self)
            }
        }
    };
}

impl_into_any_transform!(Mode1Tf, Mode1);
impl_into_any_transform!(Mode2Tf, Mode2);
impl_into_any_transform!(Mode3Tf, Mode3);
impl_into_any_transform!(Mode4Tf, Mode4);
impl_into_any_transform!(Mode5Tf, Mode5);
impl_into_any_transform!(Mode7Tf, Mode7);
impl_into_any_transform!(Mode8Tf, Mode8);
impl_into_any_transform!(Mode9Tf, Mode9);
