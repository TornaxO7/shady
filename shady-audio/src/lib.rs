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
//! You mainly interact with [ShadyAudio] and start there by clicking on the link.
//!
//! # Example
//! This example basically contains the full API:
//!
//! ```rust
//! use std::num::NonZeroUsize;
//!
//! use shady_audio::{ShadyAudio, fetcher::DummyFetcher, config::ShadyAudioConfig};
//!
//! let mut audio = {
//!     let fetcher = DummyFetcher::new();
//!     let config = ShadyAudioConfig::default();
//!
//!     ShadyAudio::new(fetcher, config)
//! };
//!
//! // Retrieve a spline which you can use, to get any points from the frequancy bands of your audio fetcher.
//! // `shady-audio` will take care of the rest. Let it be
//! //   - gravity effect
//! //   - smooth transition
//! //   - etc.
//! let spline = audio.get_spline();
//!
//! // All relevant points of the spline are stored within the range [0, 1].
//! // Since we're currently using the [DummyFetcher] our spline equals the function `f(x) = 0`:
//! assert_eq!(spline.sample(0.0), Some(0.0));
//! assert_eq!(spline.sample(0.5), Some(0.0));
//! // actually for some reason, `splines::Spline` returns `None` here and I don't know why ._.
//! assert_eq!(spline.sample(1.0), None);
//!
//! // Any other value inside [0, 1] is fine:
//! assert_eq!(spline.sample(0.123456789), Some(0.0));
//! ```
pub mod config;
pub mod fetcher;

// mod audio_spline;
// mod engine;
mod equalizer;
mod fft;
// mod magnitude;
// mod ring_buffer;
// mod timer;

// pub use audio_spline::FreqSpline;
pub use cpal;

use config::{ConfigError, ShadyAudioConfig};
use cpal::SampleRate;
use equalizer::Equalizer;
use fetcher::Fetcher;
use fft::FftCalculator;
use std::{num::NonZeroUsize, ops::Range};

type Hz = u32;

// The starting frequency from where the spline will collect/create its points.
// const START_FREQ: Hz = 20;
// The ending frequency from where the spline will stop collecting/create its points.
// const END_FREQ: Hz = 15_000

struct State {
    pub amount_bars: usize,
    pub sample_rate: SampleRate,
    pub freq_range: Range<Hz>,
    pub sensitivity: f32,
}

/// The main struct to interact with the crate.
pub struct ShadyAudio {
    state: State,
    sample_buffer: Vec<f32>,

    fetcher: Box<dyn Fetcher>,
    fft: FftCalculator,
    equalizer: Equalizer,
    // timer: Timer,
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
            None,
        );

        Ok(Self {
            state,
            fetcher,
            fft,
            sample_buffer,
            equalizer,
        })
    }

    /// Returns you the latest "frequency spline" which describes the presence of each frequency.
    /// The spline basically represents the curve of audio visualizers which are using a bar chart
    /// like [`shady-cli`](https://github.com/TornaxO7/shady/tree/main/shady-cli).
    // pub fn get_spline(&mut self) -> &FreqSpline {
    //     let magnitudes = match self.timer.ease_time() {
    //         Some(ease_time) => self.magnitudes.update_with_ease(ease_time),
    //         None => {
    //             let data_buf = self.fft_input.as_mut_slice();

    //             self.fetcher.fetch_snapshot(data_buf);
    //             let fft_out = self.fft.process(data_buf);
    //             self.magnitudes.update_magnitudes(fft_out)
    //         }
    //     };
    //     self.spline.update(magnitudes);

    //     &self.spline
    // }

    pub fn get_bars(&mut self) -> &[f32] {
        // TODO: Fetcher fetcht zu große blöcke, aber dafür in kleineren zeitslots
        self.fetcher.fetch_samples(&mut self.sample_buffer);
        let fft_out = self.fft.process(&self.sample_buffer);
        let bars = self.equalizer.process(fft_out);

        self.sample_buffer.clear();

        bars
    }

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

    // /// Let's you update the internal config if you'd like to change something.
    // pub fn update_config(&mut self, config: ShadyAudioConfig) {
    //     self.timer.set_refresh_time(config.refresh_time);
    // }
}
