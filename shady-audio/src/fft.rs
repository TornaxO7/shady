use crate::{END_FREQ, START_FREQ};
use realfft::{num_complex::Complex32, RealFftPlanner};
use ringbuffer::{AllocRingBuffer, RingBuffer};

pub const FFT_INPUT_SIZE: usize = 44_100;
pub const FFT_OUTPUT_SIZE: usize = FFT_INPUT_SIZE / 2 + 1;

const AMOUNT_HIGHEST_MAGNITUDES: usize = 10;
const GRAVITY_DECAY: f32 = 0.9;

pub struct FftCalculator {
    planner: RealFftPlanner<f32>,
    hann_window: Box<[f32]>,
    scratch_buffer: Box<[Complex32; FFT_INPUT_SIZE]>,

    magnitudes: Box<[f32; FFT_OUTPUT_SIZE]>,
    fft_output: Box<[Complex32; FFT_OUTPUT_SIZE]>,

    highest_magnitudes: AllocRingBuffer<f32>,

    index_of_last_max: usize,
    index_of_curr_max: usize,
}

impl FftCalculator {
    pub fn new() -> Self {
        let scratch_buffer = Box::new([Complex32::ZERO; FFT_INPUT_SIZE]);
        let hann_window = apodize::hanning_iter(FFT_INPUT_SIZE)
            .map(|hanning| hanning as f32)
            .collect::<Vec<f32>>()
            .into_boxed_slice();
        let fft_output = Box::new([Complex32::ZERO; FFT_OUTPUT_SIZE]);
        let magnitudes = Box::new([0.; FFT_OUTPUT_SIZE]);

        // invariant checking
        debug_assert_eq!(FFT_INPUT_SIZE, scratch_buffer.len());
        debug_assert_eq!(scratch_buffer.len(), hann_window.len());

        debug_assert_eq!(FFT_OUTPUT_SIZE, fft_output.len());
        debug_assert_eq!(fft_output.len(), magnitudes.len());

        Self {
            planner: RealFftPlanner::new(),
            scratch_buffer,
            hann_window,
            fft_output,
            magnitudes,

            highest_magnitudes: AllocRingBuffer::new(AMOUNT_HIGHEST_MAGNITUDES),
            index_of_last_max: 0,
            index_of_curr_max: 0,
        }
    }

    pub fn process(&mut self, data: &mut [f32]) -> &[f32] {
        debug_assert!(!data.is_empty());

        for (val, window) in data.iter_mut().zip(self.hann_window.iter()) {
            *val *= window;
        }

        self.calc_fft(data);
        self.calc_magnitudes();
        self.normalize_magnitudes();

        self.magnitudes.as_ref()
    }

    fn calc_fft(&mut self, data: &[f32]) {
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
            self.magnitudes[i] = mag.max(self.magnitudes[i] * GRAVITY_DECAY);

            if START_FREQ <= i && i <= END_FREQ {
                if mag > max {
                    max = mag;
                    self.index_of_curr_max = i;
                }
            }
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
        for mag in self.magnitudes.iter_mut() {
            // It could be that the current audio is playing a very quiet section (for example at the end of a YT-Video), so the average goes down to 0
            // => avoid dividing by zero
            if max < 1. {
                *mag *= max;
            } else {
                *mag /= max;
            }
        }

        self.index_of_last_max = self.index_of_curr_max;
    }
}
