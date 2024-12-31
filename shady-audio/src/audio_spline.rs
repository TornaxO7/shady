use crate::{fft, START_FREQ};
use cpal::SampleRate;
use splines::{Key, Spline};

const EXP_BASE: f32 = 1.06;

pub struct FreqSpline {
    spline: Spline<f32, f32>,
    freq_step: f32,
    amount_points: usize,
}

impl FreqSpline {
    pub fn new(sample_rate: SampleRate) -> Self {
        let freq_step = sample_rate.0 as f32 / fft::FFT_INPUT_SIZE as f32;

        let amount_points = {
            let mut counter: usize = 0;
            let mut end_freq = (START_FREQ as f32 + freq_step) * EXP_BASE;

            while end_freq < fft::FFT_OUTPUT_SIZE as f32 {
                counter += 1;
                end_freq = (START_FREQ as f32 + counter as f32 * freq_step)
                    * EXP_BASE.powi(counter as i32);
            }

            counter
        };

        let spline = {
            let mut spline = Spline::from_vec(vec![]);

            let step = 1. / (amount_points - 1) as f32; // `-1` in order to reach `1.`

            for i in 0..amount_points as usize {
                let x = i as f32 * step;
                let key = Key::new(x, 0.0, splines::Interpolation::Linear);
                spline.add(key);
            }
            spline
        };

        Self {
            spline,
            amount_points,
            freq_step,
        }
    }

    pub fn update(&mut self, magnitudes: &[f32]) {
        debug_assert_eq!(magnitudes.len(), fft::FFT_OUTPUT_SIZE);

        let mut start_freq = START_FREQ as f32;
        let mut end_freq = (start_freq + self.freq_step) * EXP_BASE;

        for i in 0..self.amount_points {
            let start = start_freq as usize;
            let end = end_freq as usize;

            let value = magnitudes[start..end].iter().sum::<f32>() / (end - start) as f32;

            start_freq = end_freq;
            end_freq = {
                let i_next = i + 1;

                let next_general_end = (START_FREQ as f32) + i_next as f32 * self.freq_step;
                let human_hear_end = next_general_end * EXP_BASE.powi(i_next as i32);

                human_hear_end.ceil() // guarantee that end_freq > start_freq
            };

            *self.spline.get_mut(i as usize).unwrap().value = value;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.spline.is_empty()
    }

    pub fn sample(&self, t: f32) -> Option<f32> {
        self.spline.sample(t)
    }
}
