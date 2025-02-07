//! Each struct here can be used to fetch the audio data from various sources.
//! Pick the one you need to fetch from.
mod dummy;
mod system_audio;

use cpal::SampleRate;
pub use dummy::DummyFetcher;
pub use system_audio::SystemAudio as SystemAudioFetcher;

/// Interface for all structs (fetchers) which are listed in the [fetcher module](crate::fetcher).
pub trait Fetcher {
    /// **Replaces** the content of `buf` with the data from the given fetcher.
    fn fetch_samples(&mut self, buf: &mut Vec<f32>);

    fn sample_rate(&self) -> SampleRate;
}
