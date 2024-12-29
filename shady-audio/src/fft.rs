use realfft::{num_complex::Complex32, RealFftPlanner};
use ringbuffer::{AllocRingBuffer, RingBuffer};
use tracing::debug;

use crate::{END_FREQ, SAMPLE_RATE, START_FREQ};

pub const FFT_OUTPUT_SIZE: usize = SAMPLE_RATE / 2 + 1;
const FFT_INPUT_SIZE: usize = SAMPLE_RATE;

const AMOUNT_HIGHEST_MAGNITUDES: usize = 1000 / 60;
const GRAVITY_DECAY: f32 = 0.975;

pub struct FftCalculator {
    planner: RealFftPlanner<f32>,
    scratch_buffer: Box<[Complex32; FFT_INPUT_SIZE]>,
    fft_output: Box<[Complex32; FFT_OUTPUT_SIZE]>,
    magnitudes: Box<[f32; FFT_OUTPUT_SIZE]>,
    hann_window: Box<[f32; FFT_INPUT_SIZE]>,

    highest_magnitudes: AllocRingBuffer<f32>,
    index_of_last_max: usize,
    index_of_curr_max: usize,
}

impl FftCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn process(&mut self, data: &mut [f32]) -> &[f32] {
        self.process_inner(data, true)
    }

    fn process_inner(&mut self, data: &mut [f32], window: bool) -> &[f32] {
        debug_assert_eq!(data.len(), FFT_INPUT_SIZE);

        if window {
            for (val, window) in data.iter_mut().zip(self.hann_window.iter()) {
                *val *= window;
            }
        }

        self.calc_fft(data);
        self.calc_magnitudes();
        self.normalize_magnitudes();

        self.magnitudes.as_ref()
    }

    fn calc_fft(&mut self, data: &mut [f32]) {
        let fft = self.planner.plan_fft_forward(FFT_INPUT_SIZE);
        fft.process_with_scratch(
            &mut data.to_vec(),
            self.fft_output.as_mut_slice(),
            self.scratch_buffer.as_mut_slice(),
        )
        .unwrap();
    }

    // Calculates the magnitudes out of the fft output
    fn calc_magnitudes(&mut self) {
        let mut max = f32::MIN;
        for (i, val) in self.fft_output.iter().enumerate() {
            let mag = val.norm();

            if START_FREQ <= i && i <= END_FREQ {
                if mag > max {
                    max = mag;
                    self.index_of_curr_max = i;
                }
            }

            self.magnitudes[i] = mag.max(self.magnitudes[i] * GRAVITY_DECAY);
        }

        self.highest_magnitudes.push(max);
    }

    fn current_highest_magnitude(&self) -> f32 {
        let curr_avg =
            self.highest_magnitudes.iter().sum::<f32>() / AMOUNT_HIGHEST_MAGNITUDES as f32;

        curr_avg
    }

    fn normalize_magnitudes(&mut self) {
        let max = self.current_highest_magnitude();
        for (i, mag) in self.magnitudes.iter_mut().enumerate() {
            // It could be that the current audio is playing a very quiet section, so the average goes down to 0
            // => avoid dividing by zero`
            if max < 1. {
                let sound_is_playing = (*mag - max).abs() > 0.001;
                if i == self.index_of_last_max && sound_is_playing {
                    *mag = 1.;
                }
            } else {
                *mag /= max;
            }
        }

        self.index_of_last_max = self.index_of_curr_max;
    }
}

impl Default for FftCalculator {
    fn default() -> Self {
        let hann_window = {
            let mut hann_window = Box::new([0.; FFT_INPUT_SIZE]);

            for (i, val) in apodize::hanning_iter(FFT_INPUT_SIZE)
                .map(|x| x as f32)
                .enumerate()
            {
                hann_window[i] = val;
            }

            hann_window
        };

        Self {
            planner: RealFftPlanner::new(),
            scratch_buffer: Box::new([Complex32::ZERO; FFT_INPUT_SIZE]),
            fft_output: Box::new([Complex32::ZERO; FFT_OUTPUT_SIZE]),
            magnitudes: Box::new([0.; FFT_OUTPUT_SIZE]),
            hann_window,
            index_of_last_max: 0,
            index_of_curr_max: 0,
            highest_magnitudes: AllocRingBuffer::from([1.; AMOUNT_HIGHEST_MAGNITUDES]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// aka: 20Khz
    ///
    /// Check if the magnitudes are correctly calculated. In this case, we just want the magnitude
    /// for 20kHz to have a spike
    mod highest_freq {

        use super::*;

        fn sin(x: f32) -> f32 {
            (2. * std::f32::consts::PI * 20_000. * x).sin()
        }

        #[test]
        fn test() {
            let mut fft = FftCalculator::new();

            let mut data = {
                let mut data: [f32; FFT_INPUT_SIZE] = [0.; FFT_INPUT_SIZE];

                for i in 0..FFT_INPUT_SIZE {
                    let frac = i as f32 / SAMPLE_RATE as f32;

                    data[i] = sin(frac);
                }

                data
            };

            let magnitudes = fft.process_inner(&mut data, false);
            for (i, &mag) in magnitudes.iter().enumerate() {
                if i != 20_000 {
                    assert!(mag < 0.5, "Non-20kHz frequency has magnitude of: {}", mag);
                } else {
                    assert!(mag >= 1., "20kHz frequency has magnitude of: {}", mag);
                }
            }
        }
    }
}
