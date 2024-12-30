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
mod fetcher;
mod fft;

use cpal::{SampleRate, StreamError, SupportedStreamConfig};
use fetcher::SystemAudio;
use fft::FftCalculator;
use splines::{Key, Spline};
use tracing::debug;

type Hz = usize;

const SAMPLE_RATE: usize = 44_100;
const REQUIRED_SAMPLE_RATE: SampleRate = SampleRate(SAMPLE_RATE as u32); // unit: Hz, about audio stream quality

const START_FREQ: Hz = 20;
const END_FREQ: Hz = 20_000;
const EXP_BASE: f32 = 1.06;

trait Data {
    fn fetch_snapshot(&mut self, buf: &mut [f32]);
}

/// The main struct to interact with the crate.
pub struct ShadyAudio {
    input_snapshot: Box<[f32; SAMPLE_RATE]>,

    data: Box<dyn Data>,
    fft: FftCalculator,
    spline: Spline<f32, f32>,
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
        stream_config: Option<&SupportedStreamConfig>,
        error_callback: E,
    ) -> Self
    where
        E: FnMut(StreamError) + Send + 'static,
    {
        let data = SystemAudio::boxed(device, stream_config, error_callback);
        Self {
            data,
            fft: FftCalculator::new(),
            input_snapshot: Box::new([0.; SAMPLE_RATE]),
            spline: default_spline(),
        }
    }

    pub fn fetch_block(&mut self) -> &Spline<f32, f32> {
        let magnitudes = {
            self.data.fetch_snapshot(self.input_snapshot.as_mut_slice());
            self.fft.process(self.input_snapshot.as_mut_slice())
        };

        let mut start_freq = START_FREQ as f32;
        let mut end_freq = start_freq * EXP_BASE;
        for i in 0..self.spline.len() {
            let start = start_freq as usize;
            let end = end_freq as usize;

            let value = magnitudes[start..end]
                .iter()
                .fold(f32::MIN, |a, &b| a.max(b));

            start_freq = end_freq;
            end_freq = (end_freq * EXP_BASE).min(fft::FFT_OUTPUT_SIZE as f32);

            *self.spline.get_mut(i).unwrap().value = value;
        }

        &self.spline
    }
}

fn default_spline() -> Spline<f32, f32> {
    let mut spline = Spline::from_vec(vec![]);

    let amount_points = (fft::FFT_OUTPUT_SIZE as f32 / START_FREQ as f32)
        .log(EXP_BASE)
        .ceil();
    let step = 1. / (amount_points - 1.); // `-1` in order to reach `1.`

    for i in 0..amount_points as usize {
        let x = i as f32 * step;
        let key = Key::new(x, 0.0, splines::Interpolation::Cosine);
        spline.add(key);
    }
    spline
}
