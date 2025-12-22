use crate::audio::AudioSamples;

use super::listener::AudioListener;
use eyre::{Result, eyre};
use rodio::{Decoder, OutputStream, Sink, Source};
use std::{
    fs::File,
    io::BufReader,
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};

/// Custom source wrapper that forwards samples to a listener
struct MonitoredSource<S>
where
    S: Source<Item = u16>,
{
    source: S,
    listener: Arc<Mutex<dyn AudioListener + Send>>,
    sample_rate: u32,
    channels: u16,
    buffer: Vec<u16>,
    buffer_size: usize,
}

impl<S> MonitoredSource<S>
where
    S: Source<Item = u16>,
{
    fn new(source: S, listener: Arc<Mutex<dyn AudioListener + Send>>, buffer_size: usize) -> Self {
        let sample_rate = source.sample_rate();
        let channels = source.channels();
        Self {
            source,
            listener,
            sample_rate,
            channels,
            buffer: Vec::with_capacity(buffer_size),
            buffer_size,
        }
    }
}

impl<S> Iterator for MonitoredSource<S>
where
    S: Source<Item = u16>,
{
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        let sample = self.source.next()?;
        self.buffer.push(sample);

        // Forward buffered samples to listener when buffer is full
        if self.buffer.len() >= self.buffer_size {
            if let Ok(mut listener) = self.listener.lock() {
                let samples = AudioSamples::new(
                    core::mem::take(&mut self.buffer),
                    self.sample_rate,
                    self.channels,
                );
                listener.on_samples(&samples);
            }
        }

        Some(sample)
    }
}

impl<S> Source for MonitoredSource<S>
where
    S: Source<Item = u16>,
{
    fn current_frame_len(&self) -> Option<usize> {
        self.source.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.source.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.source.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.source.total_duration()
    }
}

/// A struct that handles audio playback with listener support
pub struct Playback {
    _stream: OutputStream,
    sink: Sink,
    listener: Option<Arc<Mutex<dyn AudioListener + Send>>>,
}

impl Playback {
    /// Creates a new Playback instance
    pub fn new() -> Result<Self> {
        let (stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| eyre!("Failed to create output stream: {}", e))?;

        let sink =
            Sink::try_new(&stream_handle).map_err(|e| eyre!("Failed to create sink: {}", e))?;

        Ok(Self { _stream: stream, sink, listener: None })
    }

    /// Sets an audio listener to receive samples during playback
    pub fn set_listener(&mut self, listener: Arc<Mutex<dyn AudioListener + Send>>) {
        self.listener = Some(listener);
    }

    /// Loads and plays an audio file
    pub fn play<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let file =
            File::open(path.as_ref()).map_err(|e| eyre!("Failed to open audio file: {}", e))?;
        let source = Decoder::new(BufReader::new(file))
            .map_err(|e| eyre!("Failed to decode audio file: {}", e))?;

        // Convert to f32 samples
        let source = source.convert_samples();

        // If we have a listener, wrap the source to forward samples
        if let Some(listener) = &self.listener {
            let buffer_size = listener.lock().unwrap().buffer_size();
            let monitored_source = MonitoredSource::new(source, Arc::clone(listener), buffer_size);
            self.sink.append(monitored_source);
        } else {
            self.sink.append(source);
        }

        Ok(())
    }

    /// Pauses playback
    pub fn pause(&self) {
        self.sink.pause();
    }

    /// Resumes playback
    pub fn resume(&self) {
        self.sink.play();
    }

    /// Stops playback
    pub fn stop(&self) {
        self.sink.stop();
    }

    /// Returns true if playback is paused
    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }

    /// Returns true if there's nothing queued in the sink
    pub fn is_empty(&self) -> bool {
        self.sink.empty()
    }

    /// Blocks until all queued audio has finished playing
    pub fn wait_until_done(&self) {
        self.sink.sleep_until_end();
    }

    /// Gets the current volume (0.0 to 1.0+)
    pub fn volume(&self) -> f32 {
        self.sink.volume()
    }

    /// Sets the volume (0.0 to 1.0+)
    pub fn set_volume(&self, volume: f32) {
        self.sink.set_volume(volume);
    }
}

impl Default for Playback {
    fn default() -> Self {
        Self::new().expect("Failed to create default Playback")
    }
}
