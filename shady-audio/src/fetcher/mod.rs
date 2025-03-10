//! Each struct here can be used to fetch the audio data from various sources.
//! Pick the one you need to fetch from.
mod dummy;
mod system_audio;

use cpal::SampleRate;

pub use dummy::DummyFetcher;
pub use system_audio::{SystemAudio as SystemAudioFetcher, SystemAudioError};

/// Interface for all structs (fetchers) which are listed in the [fetcher module](crate::fetcher).
pub trait Fetcher {
    fn fetch_samples(&mut self, buf: &mut [f32]);

    /// Returns the sample rate of the fetcher/audio source.
    fn sample_rate(&self) -> SampleRate;
}
