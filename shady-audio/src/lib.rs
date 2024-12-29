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

use splines::{Key, Spline};
use std::sync::{Arc, Mutex};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleFormat, SampleRate, StreamError, SupportedStreamConfig,
};
use fft::FftCalculator;
use ringbuffer::{AllocRingBuffer, RingBuffer};
use tracing::debug;

const SAMPLE_RATE: usize = 44_100;
const REQUIRED_SAMPLE_RATE: SampleRate = SampleRate(SAMPLE_RATE as u32); // unit: Hz, about audio stream quality

const EXP_BASE: f32 = 1.06;

/// The main struct to interact with the crate.
pub struct ShadyAudio {
    input_samples_buffer: Arc<Mutex<AllocRingBuffer<f32>>>,
    input_snapshot: [f32; SAMPLE_RATE],

    fft: FftCalculator,
    spline: Spline<f32, f32>,

    _stream: cpal::Stream,
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

        let input_samples_buffer: Arc<Mutex<AllocRingBuffer<f32>>> =
            Arc::new(Mutex::new(AllocRingBuffer::new(SAMPLE_RATE)));

        let stream = device
            .build_input_stream(
                &stream_config,
                {
                    let buffer = input_samples_buffer.clone();
                    move |input_samples: &[f32], _: &cpal::InputCallbackInfo| {
                        let mut buf = buffer.lock().unwrap();
                        buf.extend(input_samples.iter().cloned());
                    }
                },
                error_callback,
                None,
            )
            .expect("Start audio listening");

        stream.play().expect("Start listening to audio");

        let spline = {
            let mut spline = Spline::from_vec(vec![]);

            let amount_points = (fft::FFT_OUTPUT_SIZE as f32 / 20.).log(EXP_BASE).ceil();
            let step = 1. / amount_points;

            for i in 0..amount_points as usize {
                let x = i as f32 * step;
                let key = Key::new(x, 0.0, splines::Interpolation::CatmullRom);
                spline.add(key);
            }
            spline
        };

        Self {
            input_samples_buffer,
            _stream: stream,
            fft: FftCalculator::new(),
            input_snapshot: [0.; SAMPLE_RATE],
            spline,
        }
    }

    pub fn fetch_block(&mut self) -> &Spline<f32, f32> {
        let magnitudes = {
            {
                let audio = self.input_samples_buffer.lock().unwrap();

                for i in 0..SAMPLE_RATE {
                    self.input_snapshot[i] = *audio.get(i).unwrap_or(&0.0);
                }
            }

            self.fft.process(self.input_snapshot.as_mut_slice())
        };

        let mut start_freq = 20.;
        let mut end_freq = start_freq * EXP_BASE;
        for i in 0..self.spline.len() {
            let value = magnitudes[start_freq as usize..end_freq as usize]
                .iter()
                .fold(f32::MIN, |a, &b| a.max(b));

            start_freq = end_freq;
            end_freq = (end_freq * EXP_BASE).min(fft::FFT_OUTPUT_SIZE as f32);

            *self.spline.get_mut(i).unwrap().value = value;
        }

        &self.spline
    }
}

fn default_output_config(device: &cpal::Device) -> SupportedStreamConfig {
    let mut matching_configs: Vec<_> = device
        .supported_output_configs()
        .expect("Get supported output configs of device")
        .filter(|entry| {
            entry.channels() == 1
                && entry.sample_format() == SampleFormat::F32
                && entry.min_sample_rate() <= REQUIRED_SAMPLE_RATE
        })
        .collect();

    matching_configs.sort_by(|a, b| a.cmp_default_heuristics(b));

    match matching_configs.into_iter().next() {
        Some(config) => {
            debug!("Found matching output config: {:?}", config);
            config.with_max_sample_rate()
        }
        None => {
            debug!("Didn't find matching output config. Fallback to default_output_config.");
            device
                .default_output_config()
                .expect("Get default output config for device")
        }
    }
}
