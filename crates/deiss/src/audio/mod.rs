//! Audio playback module with listener support

mod console_listener;
mod listener;
mod playback;

use std::ops::Deref;

pub use console_listener::ConsoleAudioListener;
pub use listener::AudioListener;
pub use playback::Playback;

pub struct AudioSamples {
    /// Slice of f32 audio samples (interleaved if stereo)
    samples: Vec<f32>,

    /// The sample rate of the audio
    sample_rate: u32,

    /// Number of audio channels (1 for mono, 2 for stereo)
    channels: u16,
}

impl AudioSamples {
    pub fn new(samples: Vec<f32>, sample_rate: u32, channels: u16) -> Self {
        AudioSamples {
            samples,
            sample_rate,
            channels,
        }
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn samples(&self) -> &[f32] {
        &self.samples
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn channels(&self) -> u16 {
        self.channels
    }
}

impl Deref for AudioSamples {
    type Target = [f32];

    fn deref(&self) -> &Self::Target {
        &self.samples
    }
}
