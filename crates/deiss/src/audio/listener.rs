use crate::audio::AudioSamples;

/// Trait for listening to audio samples during playback
pub trait AudioListener {
    /// Called when audio samples are read from the audio file
    fn on_samples(&mut self, samples: &AudioSamples);
}
