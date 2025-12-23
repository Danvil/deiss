use crate::{painter::*, utils::Minstd};
use core::ops;
use std::collections::HashMap;

pub struct ModeBlueprintLibrary {
    pub gf: [f32; 6],
    pub mode_info: HashMap<ModeId, ModeBlueprint>,
}

impl ModeBlueprintLibrary {
    pub fn new(g: &mut Globals) -> Self {
        let gf = core::array::from_fn(|_| g.rand.next_01_prom() * 0.01 + 0.02);

        let mut mode_info = HashMap::new();

        mode_info.insert(
            ModeId(1),
            ModeBlueprint {
                effect_freq: [220, 150, 10, 680, 4, 170, 400, 0].into(),
                solar_max: 800,
                center_dwindle: 1.0,
                effect_count: [1, 2],
                motion_dampened: true,
                tf_gen: Box::new(|rand: &mut _| mode_1_tf(rand).into()),
            },
        );

        mode_info.insert(
            ModeId(2),
            ModeBlueprint {
                effect_freq: [750, 500, 750, 750, 0, 0, 0, 0].into(),
                solar_max: 35,
                center_dwindle: 1.0,
                effect_count: [1, 5],
                motion_dampened: true,
                tf_gen: Box::new(|rand: &mut _| mode_2_tf(rand).into()),
            },
        );

        mode_info.insert(
            ModeId(3),
            ModeBlueprint {
                effect_freq: [100, 100, 100, 500, 10, 0, 300, 0].into(),
                solar_max: 60,
                center_dwindle: 0.99,
                effect_count: [1, 2],
                motion_dampened: false,
                tf_gen: Box::new(|rand: &mut _| mode_3_tf(rand).into()),
            },
        );

        mode_info.insert(
            ModeId(4),
            ModeBlueprint {
                effect_freq: [500, 100, 100, 100, 30, 0, 0, 0].into(),
                solar_max: 34,
                center_dwindle: 0.98,
                effect_count: [1, 2],
                motion_dampened: true,
                tf_gen: Box::new(|rand: &mut _| mode_4_tf(rand).into()),
            },
        );

        mode_info.insert(
            ModeId(5),
            ModeBlueprint {
                effect_freq: [100, 350, 100, 500, 15, 180, 500, 0].into(),
                solar_max: 60,
                center_dwindle: 0.99,
                effect_count: [1, 2],
                motion_dampened: true,
                tf_gen: Box::new(|rand: &mut _| {
                    let has_nuclide_effect = true; // FIXME
                    mode_5_tf(has_nuclide_effect, rand).into()
                }),
            },
        );

        // mode_info.insert(
        //     ModeId(6),
        //     ModeBlueprint {
        //         effect_freq: [400, 120, 200, 0, 0, 0, 0, 0].into(),
        //         solar_max: 60,
        //         center_dwindle: 1.0,
        //         effect_count: [1, 2],
        //         motion_dampened: false,
        //     },
        // );

        mode_info.insert(
            ModeId(7),
            ModeBlueprint {
                effect_freq: [50, 200, 0, 300, 0, 600, 350, 0].into(),
                solar_max: 65,
                center_dwindle: 0.985,
                effect_count: [1, 2],
                motion_dampened: true,
                tf_gen: Box::new(|rand: &mut _| mode_7_tf(rand).into()),
            },
        );

        mode_info.insert(
            ModeId(8),
            ModeBlueprint {
                effect_freq: [150, 150, 150, 150, 25, 0, 0, 0].into(),
                solar_max: 60,
                center_dwindle: 0.96,
                effect_count: [1, 2],
                motion_dampened: true,
                tf_gen: Box::new(|rand: &mut _| mode_8_tf(rand).into()),
            },
        );

        mode_info.insert(
            ModeId(9),
            ModeBlueprint {
                effect_freq: [450, 200, 50, 200, 0, 100, 200, 0].into(),
                solar_max: 50,
                center_dwindle: 0.985,
                effect_count: [1, 2],
                motion_dampened: true,
                tf_gen: Box::new(|rand: &mut _| mode_9_tf(rand).into()),
            },
        );

        // mode_info.insert(
        //     ModeId(10),
        //     ModeBlueprint {
        //         effect_freq: [150, 20, 80, 0, 0, 80, 0, 0].into(),
        //         solar_max: 0,
        //         center_dwindle: 1.0,
        //         effect_count: [0, 2],
        //         motion_dampened: true,
        //     },
        // );

        // mode_info.insert(
        //     ModeId(11),
        //     ModeBlueprint {
        //         effect_freq: [360, 200, 230, 550, 10, 330, 150, 0].into(),
        //         solar_max: 750,
        //         center_dwindle: 1.0,
        //         effect_count: [0, 4],
        //         motion_dampened: true,
        //     },
        // );

        // mode_info.insert(
        //     ModeId(12),
        //     ModeBlueprint {
        //         effect_freq: [360, 200, 230, 0, 0, 330, 0, 0].into(),
        //         solar_max: 500,
        //         center_dwindle: 0.915,
        //         effect_count: [0, 2],
        //         motion_dampened: true,
        //     },
        // );

        // mode_info.insert(
        //     ModeId(13),
        //     ModeBlueprint {
        //         effect_freq: [500, 0, 100, 0, 30, 0, 0, 0].into(),
        //         solar_max: 34,
        //         center_dwindle: 0.98,
        //         effect_count: [1, 2],
        //         motion_dampened: true,
        //     },
        // );

        // mode_info.insert(
        //     ModeId(14),
        //     ModeBlueprint {
        //         effect_freq: [500, 0, 100, 0, 30, 0, 0, 0].into(),
        //         solar_max: 34,
        //         center_dwindle: 0.98,
        //         effect_count: [1, 2],
        //         motion_dampened: true,
        //     },
        // );

        // mode_info.insert(
        //     ModeId(15),
        //     ModeBlueprint {
        //         effect_freq: [0, 0, 0, 0, 0, 200, 0, 0].into(),
        //         solar_max: 60,
        //         center_dwindle: 1.0,
        //         effect_count: [0, 1],
        //         motion_dampened: true,
        //     },
        // );

        // mode_info.insert(
        //     ModeId(16),
        //     ModeBlueprint {
        //         effect_freq: [500, 100, 100, 100, 30, 0, 0, 0].into(),
        //         solar_max: 34,
        //         center_dwindle: 0.98,
        //         effect_count: [1, 2],
        //         motion_dampened: true,
        //     },
        // );

        // for i in 17..=25 {
        //     mode_info.insert(
        //         ModeId(i),
        //         ModeBlueprint {
        //             effect_freq: [150, 150, 150, 150, 12, 0, 50, 0].into(),
        //             solar_max: 600,
        //             center_dwindle: match i {
        //                 20 | 21 | 22 | 23 => 0.98,
        //                 _ => 1.0,
        //             },
        //             effect_count: [1, 3],
        //             motion_dampened: false,
        //         },
        //     );
        // }

        // Post-processing loop to modify effect frequencies
        for (_, mi) in mode_info.iter_mut() {
            mi.effect_freq[EffectKind::Nuclide] =
                (mi.effect_freq[EffectKind::Nuclide] as f32 * 1.3).max(0.0).min(900.0) as u32;
            mi.effect_freq[EffectKind::Chasers] =
                (mi.effect_freq[EffectKind::Chasers] as i32 - 50).max(0).min(900) as u32;
            mi.effect_freq[EffectKind::Dots] = (mi.effect_freq[EffectKind::Dots] + 220).min(900);
            mi.effect_freq[EffectKind::Bar] = (mi.effect_freq[EffectKind::Bar] + 220).min(900);
            mi.effect_freq[EffectKind::Shade] = (mi.effect_freq[EffectKind::Shade] + 150).min(900);
            mi.effect_freq[EffectKind::Grid] = (mi.effect_freq[EffectKind::Grid] + 8).min(1000);
        }

        Self { gf, mode_info }
    }
}

impl ops::Index<ModeId> for ModeBlueprintLibrary {
    type Output = ModeBlueprint;

    fn index(&self, mode: ModeId) -> &Self::Output {
        &self.mode_info[&mode]
    }
}

pub trait GenerateTransform: 'static + Send + Sync {
    fn generate_transform(&self, rand: &mut Minstd) -> AnyTransform;
}

impl<F> GenerateTransform for F
where
    F: 'static + Send + Sync + for<'a> Fn(&'a mut Minstd) -> AnyTransform,
{
    fn generate_transform(&self, rand: &mut Minstd) -> AnyTransform {
        (self)(rand)
    }
}
