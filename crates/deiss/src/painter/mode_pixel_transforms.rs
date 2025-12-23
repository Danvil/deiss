use crate::{
    painter::{
        CenterTransform, DitherTurnScaleTransform, GeneralPixelTransform, ScaleF,
        TurnScaleTransform, TurnVarScaleTransform, YRoi,
    },
    utils::*,
};

const RMULT: f32 = 1.0; // 640/FXW
const PROTECTIVE_FACTOR: f32 = 1.0; // min(640/FWX, 1)

#[derive(Debug, Clone)]
pub struct Marked<T, const N: usize>(T);

impl<T: GeneralPixelTransform, const N: usize> GeneralPixelTransform for Marked<T, N> {
    fn transform(&self, p: Vec2, c: Vec2, s: Vec2) -> Vec2 {
        self.0.transform(p, c, s)
    }
}

// === Mode 1

pub type Mode1Tf = Marked<CenterTransform<DitherTurnScaleTransform>, 1>;

pub fn mode_1_tf(rand: &mut Minstd) -> Mode1Tf {
    let scale1 = 0.985 - 0.12 * rand.next_01_prom().powi(2);
    let scale2 = scale1;

    let mut turn1 = 0.01 + 0.01 * rand.next_01_prom();
    let turn2 = turn1;

    if rand.next() % 3 == 1 {
        turn1 *= -1.;
    }

    Marked(CenterTransform::new(DitherTurnScaleTransform::from_scale_turn_raw(
        [scale1, scale2],
        [turn1, turn2],
        rand,
    )))
}

// === Mode 2

pub type Mode2Tf = Marked<CenterTransform<TurnScaleTransform>, 2>;

pub fn mode_2_tf(rand: &mut Minstd) -> Mode2Tf {
    let scale = 1.00 - 0.02 * rand.next_01_prom();
    let turn = 0.02 + 0.07 * rand.next_01_prom();
    Marked(CenterTransform::new(TurnScaleTransform::from_scale_turn_raw(scale, turn, rand)))
}

// === Mode 3

pub type Mode3Tf = CenterTransform<TurnVarScaleTransform<Mode3Scale>>;

pub fn mode_3_tf(rand: &mut Minstd) -> Mode3Tf {
    let scale = 0.85 + 0.10 * rand.next_01_prom();
    let turn = 0.01 + 0.015 * rand.next_01_prom();
    CenterTransform::new(TurnVarScaleTransform::new_raw(turn, Mode3Scale(scale), rand))
}

#[derive(Debug, Clone)]
pub struct Mode3Scale(f32);

impl ScaleF for Mode3Scale {
    fn scale(&self, _: Vec2) -> f32 {
        self.0
    }
}

// === Mode 4

pub type Mode4Tf = CenterTransform<TurnVarScaleTransform<Mode4Scale>>;

pub fn mode_4_tf(rand: &mut Minstd) -> Mode4Tf {
    let turn = 0.007 + 0.02 * rand.next_01_prom();
    CenterTransform::new(TurnVarScaleTransform::new_raw(turn, Mode4Scale, rand))
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

pub type Mode5Tf = CenterTransform<TurnVarScaleTransform<Mode5Scale>>;

pub fn mode_5_tf(has_nuclide_effect: bool, rand: &mut Minstd) -> Mode5Tf {
    let turn = 0.01 + 0.03 * rand.next_01_prom();
    let f1 = 0.05 + 0.05 * rand.next_01_prom() + 0.07 * rand.next_01_prom();
    let f2 = 0.99 - 0.01 * rand.next_01_prom() - 0.02 * rand.next_01_prom();
    CenterTransform::new(TurnVarScaleTransform::new_raw(
        turn,
        Mode5Scale { has_nuclide_effect, f1, f2 },
        rand,
    ))
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

// === Mode 6

/// Mode 6 uses custom motion vectors
#[derive(Debug, Clone)]
pub struct Mode6Tf {
    c: [Vec2; 5],
    ctype: [u32; 5],
    c0: [Vec2; 5],
}

impl GeneralPixelTransform for Mode6Tf {
    fn transform(&self, p: Vec2, _: Vec2, _: Vec2) -> Vec2 {
        let mut t = Vec2::new(0., 0.);
        let mut f = 0.;

        for n in 0..5 {
            let dp = self.c[n] - p;
            let dp_norm_sq = dp.norm_squared();
            let d = 1. / (dp_norm_sq + 0.1);
            f += d;

            match self.ctype[n] {
                0 => {
                    t += self.c0[n] * d;
                }
                1 => {
                    let z = 1. / (dp_norm_sq.sqrt() + 0.01);
                    t += Vec2::new(-dp.y, dp.x) * (2. * d * z);
                }
                2 => {
                    let z = 1. / (dp_norm_sq.sqrt() + 0.01);
                    t += Vec2::new(dp.y, -dp.x) * (2. * d * z);
                }
                _ => unreachable!(),
            }
        }

        let t_scale = if f > 0.000001 { 1.9 / f } else { 0. };

        p + t * t_scale + Vec2::new(-0.1, 0.6)
    }
}

pub fn mode_6_tf(rand: &mut Minstd) -> Mode6Tf {
    const FXW: u32 = 640; // FIXME
    const YROI: YRoi = YRoi { min: 90, max: 480 - 90 }; // FIXME

    Mode6Tf {
        c: core::array::from_fn(|_| {
            Vec2::new(
                rand.next_idx(10 * FXW) as f32 * 0.1,
                YROI.min as f32 + rand.next_idx((YROI.max - YROI.min) * 10) as f32 * 0.1,
            )
        }),
        ctype: core::array::from_fn(|_| rand.next_idx(3)),
        c0: core::array::from_fn(|_| {
            let d = rand.next_idx(628) as f32 * 0.01;
            let f = 1. + rand.next_idx(80) as f32 * 0.01;
            let (sd, cd) = d.sin_cos();
            Vec2::new(cd, sd) * f
        }),
    }
}

// === Mode 7

pub type Mode7Tf = CenterTransform<TurnVarScaleTransform<Mode7Scale>>;

pub fn mode_7_tf(rand: &mut Minstd) -> Mode7Tf {
    let turn = 0.01 + 0.01 * rand.next_01_prom();
    let f1 = 0.92 + 0.01 * rand.next_01_prom();
    let f2 = 0.0006 + 0.0005 * rand.next_01_prom();
    let rand_array = (0..2345).map(|_| rand.next_idx(100) as f32 * 0.0005).collect();
    CenterTransform::new(TurnVarScaleTransform::new_raw(
        turn,
        Mode7Scale { f1, f2, rand_array },
        rand,
    ))
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

pub type Mode8Tf = CenterTransform<TurnVarScaleTransform<Mode8Scale>>;

pub fn mode_8_tf(rand: &mut Minstd) -> Mode8Tf {
    let turn = 0.05 * rand.next_01_prom();
    let f1 = rand.next_01_prom().powi(3) * 8. + 1.5;
    CenterTransform::new(TurnVarScaleTransform::new_raw(turn, Mode8Scale { f1 }, rand))
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

pub type Mode9Tf = CenterTransform<TurnVarScaleTransform<Mode9Scale>>;

pub fn mode_9_tf(rand: &mut Minstd) -> Mode9Tf {
    let turn = 0.01 + 0.03 * rand.next_01_prom();
    let f1 = 0.98 + 0.01 * rand.next_01_prom();
    let f2 = 0.0009 + 0.0012 * rand.next_01_prom();
    CenterTransform::new(TurnVarScaleTransform::new_raw(turn, Mode9Scale { f1, f2 }, rand))
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

// === Mode 10

/// Mode 10 uses custom motion vectors
#[derive(Debug, Clone)]
pub struct Mode10Tf;

impl GeneralPixelTransform for Mode10Tf {
    fn transform(&self, p: Vec2, center: Vec2, shape: Vec2) -> Vec2 {
        Vec2::new((p.x - center.x) * (1.03 + 0.03 * p.y / shape.y) + center.x, p.y * 1.04)
    }
}

pub fn mode_10_tf() -> Mode10Tf {
    Mode10Tf
}

// === Mode 11

pub type Mode11Tf = Marked<CenterTransform<DitherTurnScaleTransform>, 11>;

pub fn mode_11_tf(rand: &mut Minstd) -> Mode11Tf {
    let mut scale1 = 1.008 + 0.008 * rand.next_01_prom();
    let mut scale2 = scale1;
    let mut turn1 = 0.12 + 0.06 * rand.next_01_prom();
    let mut turn2 = turn1;
    turn1 *= -0.6;
    turn2 *= 0.1;
    scale1 *= 0.99;
    scale2 *= 1.01;
    Marked(CenterTransform::new(DitherTurnScaleTransform::from_scale_turn_raw(
        [scale1, scale2],
        [turn1, turn2],
        rand,
    )))
}

// === Mode 12

/// Mode 12 uses custom motion vectors
#[derive(Debug, Clone)]
pub struct Mode12Tf;

impl GeneralPixelTransform for Mode12Tf {
    fn transform(&self, p: Vec2, center: Vec2, _: Vec2) -> Vec2 {
        let nx = p.x - center.x;
        let dx = if nx < -0.5 {
            -(-nx).sqrt() + 0.9
        } else if nx > 0.5 {
            nx.sqrt() - 0.9
        } else {
            0.
        };
        Vec2::new(center.x + dx, p.y)
    }
}

pub fn mode_12_tf() -> Mode12Tf {
    Mode12Tf
}

// === Mode Tf enum

macro_rules! define_transform_enum {
    ($enum_name:ident { $($variant:ident($type:ty)),+ $(,)? }) => {
        #[derive(Debug, Clone)]
        pub enum $enum_name {
            $($variant($type),)+
        }

        impl GeneralPixelTransform for $enum_name {
            fn transform(&self, p: Vec2, c: Vec2, s: Vec2) -> Vec2 {
                match self {
                    $(Self::$variant(tf) => tf.transform(p, c, s),)+
                }
            }
        }

        $(
            impl From<$type> for $enum_name {
                fn from(transform: $type) -> Self {
                    Self::$variant(transform)
                }
            }
        )+
    };
}

define_transform_enum!(AnyTransform {
    Mode1(Mode1Tf),
    Mode2(Mode2Tf),
    Mode3(Mode3Tf),
    Mode4(Mode4Tf),
    Mode5(Mode5Tf),
    Mode6(Mode6Tf),
    Mode7(Mode7Tf),
    Mode8(Mode8Tf),
    Mode9(Mode9Tf),
    Mode10(Mode10Tf),
    Mode11(Mode11Tf),
    Mode12(Mode12Tf),
});
