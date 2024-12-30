use crate::{fft, DEFAULT_SAMPLE_RATE, START_FREQ};
use splines::{Key, Spline};

const EXP_BASE: f32 = 1.06;

const _INVARIANT_CHECK: () = const {
    // Currently, for calculating the frequencies, we're assuming that the sample rate
    // equals the fft size (aka the input size) because this would mean for the output
    // that the frequency represents the index of the fft output.
    assert!(DEFAULT_SAMPLE_RATE == fft::FFT_INPUT_SIZE);
};

pub struct FreqSpline {
    spline: Spline<f32, f32>,
}

impl FreqSpline {
    pub fn new() -> Self {
        let spline = {
            let mut spline = Spline::from_vec(vec![]);

            let amount_points = (fft::FFT_OUTPUT_SIZE as f32 / START_FREQ as f32)
                .log(EXP_BASE)
                .ceil();
            let step = 1. / (amount_points - 1.); // `-1` in order to reach `1.`

            for i in 0..amount_points as usize {
                let x = i as f32 * step;
                let key = Key::new(x, 0.0, splines::Interpolation::Linear);
                spline.add(key);
            }
            spline
        };

        Self { spline }
    }

    pub fn update(&mut self, magnitudes: &[f32]) {
        debug_assert_eq!(magnitudes.len(), fft::FFT_OUTPUT_SIZE);

        let mut start_freq = START_FREQ as f32;
        let mut end_freq = start_freq * EXP_BASE;

        for i in 0..self.spline.len() {
            let start = start_freq as usize;
            let end = end_freq as usize;

            let value = magnitudes[start..end].iter().sum::<f32>() / (end - start) as f32;

            start_freq = end_freq;
            end_freq = (end_freq * EXP_BASE).min(fft::FFT_OUTPUT_SIZE as f32);

            *self.spline.get_mut(i).unwrap().value = value;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.spline.is_empty()
    }

    pub fn sample(&self, t: f32) -> Option<f32> {
        self.spline.sample(t)
    }
}
