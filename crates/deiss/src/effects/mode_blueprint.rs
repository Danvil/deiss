use crate::{effects::*, utils::Minstd};

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

impl Default for ModeBlueprint {
    fn default() -> Self {
        ModeBlueprint {
            effect_freq: EffectFreq::default(),
            solar_max: 60,
            center_dwindle: 0.99,
            effect_count: [1, 2],
            motion_dampened: false,
            tf_gen: Box::new(|_: &mut _| TurnScaleTransform::from_scale_turn(1., 0.).into()),
        }
    }
}
