use crate::audio::AudioSamples;

use super::listener::AudioListener;

/// An AudioListener implementation that writes audio sample information to the console
pub struct ConsoleAudioListener {
    sample_count: usize,
}

impl ConsoleAudioListener {
    pub fn new() -> Self {
        Self { sample_count: 0 }
    }
}

impl Default for ConsoleAudioListener {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioListener for ConsoleAudioListener {
    fn on_samples(&mut self, samples: &AudioSamples) {
        self.sample_count += samples.len();

        // Print summary information
        println!(
            "Received {} samples | Sample rate: {} Hz | Channels: {} | Total samples: {}",
            samples.len(),
            samples.sample_rate(),
            samples.channels(),
            self.sample_count
        );

        // Print first few samples as an example (limit to avoid flooding console)
        if samples.len() > 0 {
            let preview_count = samples.len().min(8);
            print!("  Sample preview: [");
            for i in 0..preview_count {
                print!("{:.4}", samples[i]);
                if i < preview_count - 1 {
                    print!(", ");
                }
            }
            if samples.len() > preview_count {
                print!(", ...");
            }
            println!("]");
        }
    }
}
