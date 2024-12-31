use realfft::{num_complex::Complex32, RealFftPlanner};

pub const FFT_INPUT_SIZE: usize = 44_100; // > 44.100 Hz
pub const FFT_OUTPUT_SIZE: usize = FFT_INPUT_SIZE / 2 + 1;

pub struct FftCalculator {
    planner: RealFftPlanner<f32>,
    hann_window: Box<[f32]>,
    scratch_buffer: Box<[Complex32; FFT_INPUT_SIZE]>,

    fft_output: Box<[Complex32; FFT_OUTPUT_SIZE]>,
}

impl FftCalculator {
    pub fn new() -> Self {
        let scratch_buffer = Box::new([Complex32::ZERO; FFT_INPUT_SIZE]);
        let hann_window = apodize::hanning_iter(FFT_INPUT_SIZE)
            .map(|hanning| hanning as f32)
            .collect::<Vec<f32>>()
            .into_boxed_slice();
        let fft_output = Box::new([Complex32::ZERO; FFT_OUTPUT_SIZE]);

        // invariant checking
        debug_assert_eq!(FFT_INPUT_SIZE, scratch_buffer.len());
        debug_assert_eq!(scratch_buffer.len(), hann_window.len());

        debug_assert_eq!(FFT_OUTPUT_SIZE, fft_output.len());

        Self {
            planner: RealFftPlanner::new(),
            scratch_buffer,
            hann_window,
            fft_output,
        }
    }

    pub fn process(&mut self, data: &mut [f32]) -> &[Complex32] {
        debug_assert!(!data.is_empty());

        for (val, window) in data.iter_mut().zip(self.hann_window.iter()) {
            *val *= window;
        }

        let fft = self.planner.plan_fft_forward(FFT_INPUT_SIZE);
        fft.process_with_scratch(
            &mut data.to_vec(),
            self.fft_output.as_mut_slice(),
            self.scratch_buffer.as_mut_slice(),
        )
        .unwrap();

        self.fft_output.as_slice()
    }
}
