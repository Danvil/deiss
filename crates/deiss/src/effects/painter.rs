use crate::{
    audio::{AudioListener, AudioSamples},
    effects::{
        Effect,
        effect_dots::EffectDots,
        rgba_image::{RgbaImage, Shape2},
    },
};

pub struct Painter {
    effect: Box<dyn Effect + Send>,
    img: RgbaImage,
}

impl Painter {
    pub fn new(shape: Shape2) -> Self {
        Self {
            effect: Box::new(EffectDots::default()),
            img: RgbaImage::black(shape),
        }
    }

    pub fn image(&self) -> &RgbaImage {
        &self.img
    }
}

impl AudioListener for Painter {
    fn on_samples(&mut self, samples: &AudioSamples) {
        self.img.apply(|rgba| rgba / 2);
        self.effect.render(samples, &mut self.img);
    }
}
