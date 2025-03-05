//! # Description
//! A crate which simplifies the data management of audio sources to be easily able
//! to retrieve the frequency powers of the source.
//!
//! ### [cpal]
//!
//! This crate also re-exports [cpal] so there's no need to add [cpal] exclusively
//! to your dependency list.
//!
//! # How to get started
//! The main usage can be seen in the example below.
//! Take a look to the available methods of [ShadyAudio] if you would like to change some properties of it (like frequency range or amount of bars).
//!
//! # Example
//! ```rust
//! use std::num::NonZeroUsize;
//!
//! use shady_audio::{ShadyAudio, fetcher::DummyFetcher, config::ShadyAudioConfig};
//!
//! let mut audio = {
//!     // A fetcher feeds new samples to `ShadyAudio` which processes it
//!     let fetcher = DummyFetcher::new();
//!
//!     // configure the behaviour of `ShadyAudio`
//!     let config = ShadyAudioConfig {
//!         amount_bars: NonZeroUsize::new(10).unwrap(),
//!         ..Default::default()
//!     };
//!
//!     ShadyAudio::new(fetcher, config).unwrap()
//! };
//!
//! // just retrieve the bars.
//! // ShadyAudio takes care of the rest:
//! //   - fetching new samples from the fetcher
//! //   - normalize the values within the range [0, 1]
//! //   - etc.
//! assert_eq!(audio.get_bars().len(), 10);
//!
//! // change the amount of bars you'd like to have
//! audio.set_bars(NonZeroUsize::new(20).unwrap());
//! assert_eq!(audio.get_bars().len(), 20);
//! ```
pub mod config;
pub mod fetcher;
pub mod interpolation;

mod equalizer;
mod error;
mod fft;

type Hz = u32;

/// The minimal frequency which humans can here (roughly)
/// See: <https://en.wikipedia.org/wiki/Hearing_range>
pub const MIN_HUMAN_FREQUENCY: Hz = 20;

/// The maximal frequency which humans can here (roughly)
/// See: <https://en.wikipedia.org/wiki/Hearing_range>
pub const MAX_HUMAN_FREQUENCY: Hz = 20_000;

/// The default sample rate for a fetcher.
/// Fetchers are allowed to use this for orientation.
pub const DEFAULT_SAMPLE_RATE: SampleRate = SampleRate(44_100);

pub use cpal;

use config::ShadyAudioConfig;
use cpal::SampleRate;
use equalizer::Equalizer;
use fetcher::Fetcher;
use fft::FftCalculator;
use std::{
    num::{NonZeroU32, NonZeroUsize},
    ops::Range,
};

/// Contains all possible errors/issues with [ShadyAudio].
#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    /// Occurs, if you've set [`ShadyAudioConfig::freq_range`] to an empty range.
    ///
    /// # Example
    /// ```rust
    /// use shady_audio::{Error, config::ShadyAudioConfig};
    /// use std::num::NonZeroU32;
    ///
    /// let invalid_range = NonZeroU32::new(10).unwrap()..NonZeroU32::new(10).unwrap();
    /// assert!(invalid_range.is_empty(), "`start` and `end` are equal");
    ///
    /// let config = ShadyAudioConfig {
    ///     freq_range: invalid_range.clone(),
    ///     ..Default::default()
    /// };
    ///
    /// // the range isn't allowed to be empty!
    /// assert!(config.validate().is_err());
    /// ```
    #[error("Frequency range can't be empty but you gave: {0:?}")]
    EmptyFreqRange(Range<NonZeroU32>),
}

struct State {
    amount_bars: usize,
    sample_rate: SampleRate,
    freq_range: Range<Hz>,
    sensitivity: f32,
}

/// The main struct to interact with the crate.
pub struct ShadyAudio {
    state: State,
    sample_buffer: Vec<f32>,

    fetcher: Box<dyn Fetcher>,
    fft: FftCalculator,
    equalizer: Equalizer,
}

impl ShadyAudio {
    /// Create a new instance of this struct by providing the (audio) fetcher and the config.
    ///
    /// # Example
    /// ```
    /// use shady_audio::{ShadyAudio, fetcher::DummyFetcher, config::ShadyAudioConfig};
    ///
    /// let shady_audio = ShadyAudio::new(DummyFetcher::new(), ShadyAudioConfig::default());
    /// ```
    pub fn new(fetcher: Box<dyn Fetcher>, config: ShadyAudioConfig) -> Result<Self, Vec<Error>> {
        config.validate()?;

        let state = State {
            amount_bars: usize::from(config.amount_bars),
            sample_rate: fetcher.sample_rate(),
            freq_range: Hz::from(config.freq_range.start)..Hz::from(config.freq_range.end),
            sensitivity: 1.,
        };

        let sample_buffer = Vec::with_capacity(state.sample_rate.0 as usize);
        let fft = FftCalculator::new(state.sample_rate);
        let equalizer = Equalizer::new(
            state.amount_bars,
            state.freq_range.clone(),
            fft.size(),
            state.sample_rate,
            Some(state.sensitivity),
        );

        Ok(Self {
            state,
            fetcher,
            fft,
            sample_buffer,
            equalizer,
        })
    }

    /// Return the bars with their values.
    ///
    /// Each bar value tries to stay within the range `[0, 1]` but it could happen that there are some spikes which go above 1.
    /// However it will slowly normalize itself back to 1.
    #[inline]
    pub fn get_bars(&mut self) -> &[f32] {
        self.fetcher.fetch_samples(&mut self.sample_buffer);
        let fft_out = self.fft.process(&self.sample_buffer);
        let bars = self.equalizer.process(fft_out);

        self.sample_buffer.clear();
        bars
    }

    /// Set the length of the returned slice of [`Self::get_bars`].
    ///
    /// # Example
    /// ```
    /// use shady_audio::{ShadyAudio, fetcher::DummyFetcher, config::ShadyAudioConfig};
    /// use std::num::NonZeroUsize;
    ///
    /// let mut shady_audio = ShadyAudio::new(DummyFetcher::new(), ShadyAudioConfig::default()).unwrap();
    ///
    /// // tell `shady-audio` to compute only for four bars
    /// let amount_bars = 4;
    /// shady_audio.set_bars(NonZeroUsize::new(amount_bars).unwrap());
    ///
    /// assert_eq!(shady_audio.get_bars().len(), amount_bars);
    /// ```
    #[inline]
    pub fn set_bars(&mut self, amount_bars: NonZeroUsize) {
        self.state.amount_bars = usize::from(amount_bars);

        self.state.sensitivity = self.equalizer.sensitivity();

        self.equalizer = Equalizer::new(
            self.state.amount_bars,
            self.state.freq_range.clone(),
            self.fft.size(),
            self.state.sample_rate,
            Some(self.state.sensitivity),
        );
    }

    /// Change the fetcher.
    ///
    /// # Example
    /// ```
    /// use shady_audio::{ShadyAudio, fetcher::DummyFetcher, config::ShadyAudioConfig};
    ///
    /// let mut shady_audio = ShadyAudio::new(DummyFetcher::new(), ShadyAudioConfig::default()).unwrap();
    ///
    /// let another_fetcher = DummyFetcher::new();
    /// shady_audio.set_fetcher(another_fetcher);
    /// ```
    #[inline]
    pub fn set_fetcher(&mut self, fetcher: Box<dyn Fetcher>) {
        self.fetcher = fetcher;
    }

    /// Update the frequency range where `shady-audio` should process.
    ///
    /// Retunrs `Err` if the given range [is empty](https://doc.rust-lang.org/std/ops/struct.Range.html#method.is_empty) otherwise `Ok`.
    ///
    /// # Example
    /// ```
    /// use shady_audio::{ShadyAudio, fetcher::DummyFetcher, config::ShadyAudioConfig};
    /// use std::num::NonZeroU32;
    ///
    /// let mut shady_audio = ShadyAudio::new(DummyFetcher::new(), ShadyAudioConfig::default()).unwrap();
    ///
    /// // tell `shady-audio` to just create the bars for the frequencies from 1kHz to 15kHz.
    /// shady_audio.set_freq_range(NonZeroU32::new(1_000).unwrap()..NonZeroU32::new(15_000).unwrap()).unwrap();
    ///
    /// // empty ranges are not allowed!
    /// assert!(shady_audio.set_freq_range(NonZeroU32::new(5).unwrap()..NonZeroU32::new(5).unwrap()).is_err());
    /// ```
    #[inline]
    pub fn set_freq_range(&mut self, freq_range: Range<NonZeroU32>) -> Result<(), Error> {
        if freq_range.is_empty() {
            return Err(Error::EmptyFreqRange(freq_range));
        }
        let freq_range = u32::from(freq_range.start)..u32::from(freq_range.end);

        self.state.freq_range = freq_range;
        self.equalizer = Equalizer::new(
            self.state.amount_bars,
            self.state.freq_range.clone(),
            self.fft.size(),
            self.state.sample_rate,
            Some(self.state.sensitivity),
        );

        Ok(())
    }
}
