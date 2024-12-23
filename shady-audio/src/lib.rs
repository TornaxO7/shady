use std::{
    num::NonZeroUsize,
    sync::{Arc, Mutex, TryLockError},
};

use cpal::{
    traits::{DeviceTrait, StreamTrait},
    SampleFormat, SampleRate, StreamError, SupportedStreamConfig,
};
use realfft::{num_complex::Complex32, num_traits::Zero, RealFftPlanner};
use tracing::debug;

const REQUIRED_SAMPLE_RATE: SampleRate = SampleRate(44_100); // unit: Hz, about audio stream quality

/// The main struct.
pub struct ShadyAudio {
    input_samples_buffer: Arc<Mutex<Vec<f32>>>,

    magnitudes_buffer: Vec<f32>,

    fft_planner: RealFftPlanner<f32>,
    fft_scratch_buffer: Vec<Complex32>,
    fft_output: Vec<Complex32>,
    _stream: cpal::Stream,
}

impl ShadyAudio {
    pub fn new<E>(
        device: &cpal::Device,
        stream_config: Option<SupportedStreamConfig>,
        error_callback: E,
    ) -> Self
    where
        E: FnMut(StreamError) + Send + 'static,
    {
        let stream_config = {
            let default_output_config = default_output_config(device);
            let supported_stream_config = stream_config.as_ref().unwrap_or(&default_output_config);

            supported_stream_config.config()
        };

        let input_samples_buffer = Arc::new(Mutex::new(Vec::new()));

        let stream = device
            .build_input_stream(
                &stream_config,
                {
                    let moved_input_samples_buf = input_samples_buffer.clone();

                    move |input_samples: &[f32], _: &cpal::InputCallbackInfo| {
                        let mut buf = match moved_input_samples_buf.try_lock() {
                            Ok(raw_buf) => raw_buf,
                            Err(err) => match err {
                                TryLockError::Poisoned(_) => panic!("Poisened lock"),
                                TryLockError::WouldBlock => return,
                            },
                        };
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
            magnitudes_buffer: Vec::new(),
            fft_scratch_buffer: Vec::new(),
            fft_output: Vec::new(),
            fft_planner: RealFftPlanner::new(),
        }
    }

    pub fn fetch_magnitudes(&mut self, amount_magnitudes: NonZeroUsize) -> &[f32] {
        let mut buf = self.input_samples_buffer.lock().unwrap();

        if buf.len() % 2 != 0 {
            buf.push(0.0);
        }

        let fft = self.fft_planner.plan_fft_forward(buf.len());
        self.fft_output.resize(buf.len() / 2 + 1, Complex32::zero());
        self.fft_scratch_buffer
            .resize(fft.get_scratch_len(), Complex32::zero());

        fft.process_with_scratch(&mut buf, &mut self.fft_output, &mut self.fft_scratch_buffer)
            .unwrap();

        self.magnitudes_buffer.clear();
        let step_size = self.fft_output.len() / amount_magnitudes;
        for i in 0..amount_magnitudes.into() {
            let start = i * step_size;
            let end = (i + 1) * step_size;
            let avg_magnitude = self.fft_output[start..end]
                .iter()
                .map(|magnitude| magnitude.norm())
                .sum::<f32>();
            self.magnitudes_buffer.push(avg_magnitude);
        }

        &self.magnitudes_buffer
    }

    pub fn fetch_magnitudes_normalized(&mut self, amount_magnitudes: NonZeroUsize) -> &[f32] {
        self.fetch_magnitudes(amount_magnitudes);

        let max_magnitude = {
            let mut max = self.magnitudes_buffer[0];
            for &freq in self.magnitudes_buffer.iter() {
                if max < freq {
                    max = freq;
                }
            }
            max
        };

        if max_magnitude > 0.0 {
            for magnitude in self.magnitudes_buffer.iter_mut() {
                *magnitude /= max_magnitude;
            }
        }
        &self.magnitudes_buffer
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

#[cfg(test)]
mod tests {
    use cpal::traits::HostTrait;

    use super::*;

    #[test]
    fn expected_amount_of_channels() {
        let device = cpal::default_host().default_output_device().unwrap();

        let mut audio = ShadyAudio::new(&device, None, |err| panic!("{}", err));

        let magnitudes = audio.fetch_magnitudes(NonZeroUsize::new(10).unwrap());

        assert_eq!(magnitudes.len(), 10);
    }

    #[test]
    fn normalized_values_are_really_normalized() {
        let device = cpal::default_host().default_output_device().unwrap();

        let mut audio = ShadyAudio::new(&device, None, |err| panic!("{}", err));

        let magnitudes = audio.fetch_magnitudes_normalized(NonZeroUsize::new(10).unwrap());

        assert_eq!(magnitudes.len(), 10);

        for &magnitude in magnitudes.iter() {
            assert!(0.0 <= magnitude);
            assert!(magnitude <= 1.0);
        }
    }
}
