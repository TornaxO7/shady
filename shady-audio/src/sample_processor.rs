use cpal::SampleRate;
use realfft::{num_complex::Complex32, RealFftPlanner};

use crate::fetcher::Fetcher;

/// Prepares the samples of the fetcher for the [crate::BarProcessor].
pub struct SampleProcessor {
    planner: RealFftPlanner<f32>,
    hann_window: Box<[f32]>,

    scratch_buffer: Box<[Complex32]>,
    fft_out: Box<[Complex32]>,
    fft_in: Box<[f32]>,
    fft_in_raw: Box<[f32]>,

    fft_size: usize,
    fetcher: Box<dyn Fetcher>,
}

impl SampleProcessor {
    /// Creates a new instance with the given fetcher where the audio samples are fetched from.
    pub fn new(fetcher: Box<dyn Fetcher>) -> Self {
        let fft_size = {
            let sample_rate = fetcher.sample_rate().0;
            let factor = if sample_rate < 8_125 {
                1
            } else if sample_rate <= 16_250 {
                2
            } else if sample_rate <= 32_500 {
                4
            } else if sample_rate <= 75_000 {
                8
            } else if sample_rate <= 150_000 {
                16
            } else if sample_rate <= 300_000 {
                32
            } else {
                64
            };

            factor * 128
        };
        let fft_out_size = fft_size / 2 + 1;

        let hann_window = apodize::hanning_iter(fft_size)
            .map(|val| val as f32)
            .collect::<Vec<f32>>()
            .into_boxed_slice();

        let fft_in = vec![0.; fft_size].into_boxed_slice();
        let fft_in_raw = vec![0.; fft_size].into_boxed_slice();
        let scratch_buffer = vec![Complex32::ZERO; fft_out_size].into_boxed_slice();
        let fft_out = vec![Complex32::ZERO; fft_out_size].into_boxed_slice();

        Self {
            planner: RealFftPlanner::new(),
            hann_window,
            scratch_buffer,
            fft_out,
            fft_in,
            fft_in_raw,

            fft_size,
            fetcher,
        }
    }

    /// Tell the processor to take some samples of the fetcher and prepare them
    /// for the [crate::BarProcessor]s.
    pub fn process_next_samples(&mut self) {
        self.fetcher.fetch_samples(&mut self.fft_in_raw);

        for (i, &sample) in self.fft_in_raw.iter().enumerate() {
            self.fft_in[i] = sample * self.hann_window[i];
        }

        let fft = self.planner.plan_fft_forward(self.fft_size);
        fft.process_with_scratch(
            self.fft_in.as_mut(),
            self.fft_out.as_mut(),
            self.scratch_buffer.as_mut(),
        )
        .unwrap();
    }
}

impl SampleProcessor {
    pub(crate) fn fft_size(&self) -> usize {
        self.fft_size
    }

    pub(crate) fn fft_out(&self) -> &[Complex32] {
        &self.fft_out
    }

    pub(crate) fn sample_rate(&self) -> SampleRate {
        self.fetcher.sample_rate()
    }
}
