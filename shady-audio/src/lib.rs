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
//! You mainly interact with [ShadyAudio].
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

mod equalizer;
mod error;
mod fft;

type Hz = u32;

/// The minimal frequency which humans can here (roughly)
/// See: https://en.wikipedia.org/wiki/Hearing_range
pub const MIN_HUMAN_FREQUENCY: Hz = 20;

/// The maximal frequency which humans can here (roughly)
/// See: https://en.wikipedia.org/wiki/Hearing_range
pub const MAX_HUMAN_FREQUENCY: Hz = 20_000;

pub use cpal;

use config::{ConfigError, ShadyAudioConfig};
use cpal::SampleRate;
use equalizer::Equalizer;
use fetcher::Fetcher;
use fft::FftCalculator;
use std::{num::NonZeroUsize, ops::Range};

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
    pub fn new(
        fetcher: Box<dyn Fetcher>,
        config: ShadyAudioConfig,
    ) -> Result<Self, Vec<ConfigError>> {
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
    /// Each bar value tries to stay within the range `[0, 1]` but it could happen that there are some spikes.
    pub fn get_bars(&mut self) -> &[f32] {
        self.fetcher.fetch_samples(&mut self.sample_buffer);
        let fft_out = self.fft.process(&self.sample_buffer);
        let bars = self.equalizer.process(fft_out);

        self.sample_buffer.clear();

        bars
    }

    /// Set the length of the returned slice of [`Self::get_bars`].
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
}
