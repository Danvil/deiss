use crate::audio::AudioSamples;

/// Trait for listening to audio samples during playback
pub trait AudioListener {
    /// [on_samples] is called when this number of samples is accumulated
    fn buffer_size(&self) -> usize;

    /// Called when audio samples are read from the audio file
    fn on_samples(&mut self, samples: &AudioSamples);
}
