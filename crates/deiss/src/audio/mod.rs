//! Audio playback module with listener support

mod console_listener;
mod listener;
mod playback;

pub use console_listener::ConsoleAudioListener;
pub use listener::AudioListener;
pub use playback::Playback;
