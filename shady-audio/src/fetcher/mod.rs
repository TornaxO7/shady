//! Each struct here can be used to fetch the audio data from various sources.
//! Pick the one you need to fetch from.
mod dummy;
mod system_audio;

use cpal::SampleRate;

pub use dummy::DummyFetcher;
pub use system_audio::{
    Descriptor as SystemAudioFetcherDescriptor, SystemAudio as SystemAudioFetcher, SystemAudioError,
};

/// Interface for all structs (fetchers) which are listed in the [fetcher module](crate::fetcher).
pub trait Fetcher {
    /// Implementors should insert their samples to the beginning of `buf`
    /// and move the rest of the samples which are already in `buf` further back.
    ///
    /// In other words, you'd have to do the following:
    /// Let `n` be the amount of samples you'd like to put into `buf` (a.k.a. the new audio samples which you got).
    /// Make space in `buf` for your `n` samples in the beginnig of `buf`:
    /// 1. `buf[n..] = buf[..buf.len() - n]`.
    /// 2. `buf[..n] = your_samples[..]`
    fn fetch_samples(&mut self, buf: &mut [f32]);

    /// Returns the sample rate of the fetcher/audio source.
    fn sample_rate(&self) -> SampleRate;

    /// Returns the amount of channels which are used from the fetcher.
    fn channels(&self) -> u16;
}
