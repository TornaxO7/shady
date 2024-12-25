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
mod fft;
mod frequency_bands;

use std::sync::{Arc, Mutex};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleFormat, SampleRate, StreamError, SupportedStreamConfig,
};
use fft::FftCalculator;
use splines::{Interpolation, Key, Spline};
use tracing::debug;

const REQUIRED_SAMPLE_RATE: SampleRate = SampleRate(44_100); // unit: Hz, about audio stream quality

/// The main struct to interact with the crate.
pub struct ShadyAudio {
    input_samples_buffer: Arc<Mutex<Vec<f32>>>,

    highest_freq: f32,

    fft: FftCalculator,

    spline: Spline<f32, f32>,

    _stream: cpal::Stream,
    stream_sample_rate: usize,
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
        let default_device = cpal::default_host()
            .default_output_device()
            .expect("Default output device exists");

        let device = device.unwrap_or(&default_device);

        let stream_config = {
            let default_output_config = default_output_config(device);
            let supported_stream_config = stream_config.unwrap_or(&default_output_config);

            supported_stream_config.config()
        };

        let input_samples_buffer = Arc::new(Mutex::new(Vec::new()));

        let stream = device
            .build_input_stream(
                &stream_config,
                {
                    let moved_input_samples_buf = input_samples_buffer.clone();

                    move |input_samples: &[f32], _: &cpal::InputCallbackInfo| {
                        let mut buf = moved_input_samples_buf.lock().unwrap();
                        buf.clear();

                        buf.resize(input_samples.len(), 0.0);
                        buf.copy_from_slice(input_samples);
                    }
                },
                error_callback,
                None,
            )
            .expect("Start audio listening");

        stream.play().expect("Start listening to audio");

        Self {
            input_samples_buffer,
            _stream: stream,
            highest_freq: f32::MIN,
            spline: Spline::from_vec(Vec::new()),
            stream_sample_rate: stream_config.sample_rate.0 as usize,
            fft: FftCalculator::new(),
        }
    }

    pub fn next_spline(&mut self) -> &Spline<f32, f32> {
        self.spline.clear();

        // magnitudes is from 0 to 1
        let mut buf = self.input_samples_buffer.lock().unwrap();
        if buf.is_empty() {
            return self.spline();
        }

        let (fft_size, magnitudes) = self.fft.process(&mut buf);

        let freq_step = self.stream_sample_rate / fft_size;
        let start = 20 * fft_size / self.stream_sample_rate;
        let end = 20_000 * fft_size / self.stream_sample_rate;

        for (i, &mag) in magnitudes[start..end].iter().enumerate() {
            // normalize the frequency to [0, 1]
            let x = ((i * freq_step + start) - start) as f32 / (end - start) as f32;
            let y = mag;

            let key = Key::new(x, y, Interpolation::CatmullRom);

            self.spline.add(key);
        }

        self.spline()
    }

    fn spline(&self) -> &Spline<f32, f32> {
        &self.spline
    }

    // Reuses the old data and applies gravity effect.
    fn update_data(&mut self) -> &Spline<f32, f32> {
        self.spline()
    }
}

fn default_output_config(device: &cpal::Device) -> SupportedStreamConfig {
    let mut matching_configs: Vec<_> = device
        .supported_output_configs()
        .expect("Get supported output configs of device")
        .filter(|entry| {
            entry.channels() == 1
                && entry.sample_format() == SampleFormat::F32
                && entry.max_sample_rate() >= REQUIRED_SAMPLE_RATE
                && entry.min_sample_rate() <= REQUIRED_SAMPLE_RATE
        })
        .collect();

    matching_configs.sort_by(|a, b| a.cmp_default_heuristics(b));

    match matching_configs.into_iter().next() {
        Some(config) => {
            debug!("Found matching output config: {:?}", config);
            config.with_sample_rate(REQUIRED_SAMPLE_RATE)
        }
        None => {
            debug!("Didn't find matching output config. Fallback to default_output_config.");
            device
                .default_output_config()
                .expect("Get default output config for device")
        }
    }
}
