mod effect_dots;
mod painter;
mod rgba_image;

pub use painter::Painter;
pub use rgba_image::*;

use crate::audio::AudioSamples;

pub trait Effect {
    fn render(&self, samples: &AudioSamples, img: &mut RgbaImage);
}
