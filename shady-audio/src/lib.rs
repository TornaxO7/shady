//! This crate takes care of catching audio and giving you the desired amount of magnitudes
//! which are used for music visualizers for example.
//!
//! # Example
//! This example basically contains the full API:
//!
//! ```no_run
//! use std::num::NonZeroUsize;
//!
//! use shady_audio::ShadyAudio;
//!
//! fn main() {
//!     // use the default output and the internal heuristic config
//!     let mut audio = ShadyAudio::new(None, None, |err| panic!("{}", err));
//!
//!     // get the magnitudes with 10 entries
//!     let magnitudes = audio.fetch_magnitudes(NonZeroUsize::new(10).unwrap());
//!     assert_eq!(magnitudes.len(), 10);
//!
//!     // ... or in normalized form
//!     let norm_magnitudes = audio.fetch_magnitudes_normalized(NonZeroUsize::new(10).unwrap());
//!     for &norm_magn in norm_magnitudes {
//!         assert!(0.0 <= norm_magn);
//!         assert!(norm_magn <= 1.0);
//!     }
//!     assert_eq!(norm_magnitudes.len(), 10);
//! }
//! ```
mod audio_spline;
mod fetcher;
mod fft;

type Hz = usize;
const START_FREQ: Hz = 20;
const END_FREQ: Hz = 20_000;

use audio_spline::FreqSpline;
use cpal::{StreamError, SupportedStreamConfigRange};
use fetcher::SystemAudio;
use fft::FftCalculator;

const DEFAULT_SAMPLE_RATE: usize = fft::FFT_INPUT_SIZE;

trait Data {
    fn fetch_snapshot(&mut self, buf: &mut [f32]);
}

/// The main struct to interact with the crate.
pub struct ShadyAudio {
    fft_input: Box<[f32; fft::FFT_INPUT_SIZE]>,

    fetcher: Box<dyn Data>,
    fft: FftCalculator,
    spline: FreqSpline,
}

impl ShadyAudio {
    pub fn default_with_callback<E>(error_callback: E) -> Self
    where
        E: FnMut(StreamError) + Send + 'static,
    {
        Self::new(None, None, error_callback)
    }

    pub fn new<E>(
        device: Option<&cpal::Device>,
        stream_config_range: Option<&SupportedStreamConfigRange>,
        error_callback: E,
    ) -> Self
    where
        E: FnMut(StreamError) + Send + 'static,
    {
        Self {
            fetcher: SystemAudio::boxed(device, stream_config_range, error_callback),
            fft: FftCalculator::new(),
            fft_input: Box::new([0.; fft::FFT_INPUT_SIZE]),
            spline: FreqSpline::new(),
        }
    }

    pub fn get_spline(&mut self) -> &FreqSpline {
        let magnitudes = {
            let data_buf = self.fft_input.as_mut_slice();

            self.fetcher.fetch_snapshot(data_buf);
            self.fft.process(data_buf)
        };

        self.spline.update(magnitudes);

        &self.spline
    }
}
