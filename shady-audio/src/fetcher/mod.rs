//! Each struct here can be used to fetch the audio data from various sources.
//! Pick the one you need to fetch from.
mod dummy;
mod system_audio;

pub use dummy::DummyFetcher;
pub use system_audio::SystemAudio as SystemAudioFetcher;

/// The default (and also **required** for the time being) sample rate for all fetchers.
//
// IT's currently equal to the fft input size to have a nice step through the magnitudes of the fft output
// (1Hz equals the second mangitude-array-entry, 2Hz equals the third magnitude-array-entry and so on).
pub const DEFAULT_SAMPLE_RATE: usize = crate::fft::FFT_INPUT_SIZE;

/// Interface for all structs (fetchers) which are listed in the [fetcher module](crate::fetcher).
pub trait Fetcher {
    /// **Replaces** the content of `buf` with the data from the given fetcher.
    fn fetch_snapshot(&mut self, buf: &mut [f32]);
}
