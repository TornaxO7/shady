use realfft::{num_complex::Complex32, RealFftPlanner};
use ringbuffer::{AllocRingBuffer, RingBuffer};
use tracing::debug;

use crate::SAMPLE_RATE;

pub const FFT_OUTPUT_SIZE: usize = SAMPLE_RATE / 2 + 1;
const FFT_INPUT_SIZE: usize = SAMPLE_RATE;

const AMOUNT_HIGHEST_MAGNITUDES: usize = 60;
const GRAVITY_DECAY: f32 = 0.99;

pub struct FftCalculator {
    planner: RealFftPlanner<f32>,
    scratch_buffer: [Complex32; FFT_INPUT_SIZE],
    fft_output: [Complex32; FFT_OUTPUT_SIZE],
    magnitudes: [f32; FFT_OUTPUT_SIZE],

    highest_magnitudes: AllocRingBuffer<f32>,
}

impl FftCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn process(&mut self, data: &mut [f32]) -> &[f32] {
        debug_assert_eq!(data.len(), FFT_INPUT_SIZE);

        self.calc_fft(data);
        self.calc_magnitudes();
        self.normalize_magnitudes();

        &self.magnitudes
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
            if mag > max {
                max = mag;
            }

            self.magnitudes[i] = mag.max(self.magnitudes[i] * GRAVITY_DECAY);
        }

        self.highest_magnitudes.push(max);
    }

    fn current_highest_magnitude(&self) -> f32 {
        let curr_avg =
            self.highest_magnitudes.iter().sum::<f32>() / AMOUNT_HIGHEST_MAGNITUDES as f32;

        curr_avg.min(600f32)
    }

    fn normalize_magnitudes(&mut self) {
        let max = self.current_highest_magnitude();
        debug!("{}", max);
        for mag in self.magnitudes.iter_mut() {
            *mag /= max;
        }
    }
}

impl Default for FftCalculator {
    fn default() -> Self {
        Self {
            planner: RealFftPlanner::new(),
            scratch_buffer: [Complex32::ZERO; FFT_INPUT_SIZE],
            fft_output: [Complex32::ZERO; FFT_OUTPUT_SIZE],
            magnitudes: [0.; FFT_OUTPUT_SIZE],
            highest_magnitudes: AllocRingBuffer::from([1.; AMOUNT_HIGHEST_MAGNITUDES]),
        }
    }
}
