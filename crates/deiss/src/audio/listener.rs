/// Trait for listening to audio samples during playback
pub trait AudioListener {
    /// Called when audio samples are read from the audio file
    ///
    /// # Arguments
    /// * `samples` - Slice of f32 audio samples (interleaved if stereo)
    /// * `sample_rate` - The sample rate of the audio
    /// * `channels` - Number of audio channels (1 for mono, 2 for stereo)
    fn on_samples(&mut self, samples: &[f32], sample_rate: u32, channels: u16);
}
