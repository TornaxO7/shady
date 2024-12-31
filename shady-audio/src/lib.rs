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
//! fn main() {
//!     let mut audio = {
//!         let fetcher = DummyFetcher::boxed();
//!         let config = ShadyAudioConfig::default();
//!
//!         ShadyAudio::new(fetcher, config)
//!     };
//!
//!     // Retrieve a spline which you can use, to get any points from the frequancy bands of your audio fetcher.
//!     // `shady-audio` will take care of the rest. Let it be
//!     //   - gravity effect
//!     //   - smooth transition
//!     //   - etc.
//!     let spline = audio.get_spline();
//!
//!     // All relevant points of the spline are stored within the range [0, 1].
//!     // Since we're currently using the [DummyFetcher] our spline equals the function `f(x) = 0`:
//!     assert_eq!(spline.sample(0.0), Some(0.0));
//!     assert_eq!(spline.sample(0.5), Some(0.0));
//!     assert_eq!(spline.sample(1.0), Some(0.0));
//!
//!     // Any other value inside [0, 1] is fine:
//!     assert_eq!(spline.sample(0.123456789), Some(0.0));
//! }
//! ```
pub mod config;
pub mod fetcher;

mod audio_spline;
mod fft;
mod magnitude;
mod timer;

pub use audio_spline::FreqSpline;
pub use cpal;

use config::ShadyAudioConfig;
use fetcher::Fetcher;
use fft::FftCalculator;
use magnitude::Magnitudes;
use timer::Timer;

type Hz = usize;

// The starting frequency from where the spline will collect/create its points.
const START_FREQ: Hz = 20;
// The ending frequency from where the spline will stop collecting/create its points.
const END_FREQ: Hz = 15_000;

/// The main struct to interact with the crate.
pub struct ShadyAudio {
    fft_input: Box<[f32; fft::FFT_INPUT_SIZE]>,

    fetcher: Box<dyn Fetcher>,
    fft: FftCalculator,
    spline: FreqSpline,
    magnitudes: Magnitudes,

    timer: Timer,
}

impl ShadyAudio {
    pub fn new(fetcher: Box<dyn Fetcher>, config: ShadyAudioConfig) -> Self {
        Self {
            fetcher,
            fft: FftCalculator::new(),
            fft_input: Box::new([0.; fft::FFT_INPUT_SIZE]),
            spline: FreqSpline::new(),
            timer: Timer::new(config.refresh_time),
            magnitudes: Magnitudes::new(),
        }
    }

    pub fn get_spline(&mut self) -> &FreqSpline {
        let magnitudes = match self.timer.ease_time() {
            Some(ease_time) => self.magnitudes.update_with_ease(ease_time),
            None => {
                let data_buf = self.fft_input.as_mut_slice();

                self.fetcher.fetch_snapshot(data_buf);
                let fft_out = self.fft.process(data_buf);
                self.magnitudes.update_magnitudes(fft_out)
            }
        };
        self.spline.update(magnitudes);

        &self.spline
    }

    pub fn update_config(&mut self, config: ShadyAudioConfig) {
        self.timer.set_refresh_time(config.refresh_time);
    }
}
