use std::ops::Range;

use cpal::SampleRate;
use realfft::num_complex::Complex32;

use crate::Hz;

#[derive(Debug)]
pub struct Equalizer {
    bar_values: Box<[f32]>,
    bar_bin_indices: Box<[Range<usize>]>,

    sensitivity: f32,
}

impl Equalizer {
    pub fn new(
        amount_bars: usize,
        freq_range: Range<Hz>,
        sample_len: usize, // = fft size
        sample_rate: SampleRate,
        sensitivity: Option<f32>,
    ) -> Self {
        let bar_values = vec![0.; amount_bars].into_boxed_slice();

        let bar_bin_indices = {
            let freq_resolution = sample_rate.0 as f32 / sample_len as f32;

            // the relevant index range of the fft output which we should use for the bars
            let bin_range = Range {
                start: ((freq_range.start as f32 / freq_resolution) as usize).max(1),
                end: (freq_range.end as f32 / freq_resolution).ceil() as usize,
            };

            let amount_bins = bin_range.len();

            let weights = {
                let mut weights = Vec::new();

                for i in 0..amount_bars {
                    let weight = (i as f32 + 2.).log2().powf(20.);
                    weights.push(weight);
                }

                weights
            };

            let weight_sum = weights.iter().sum::<f32>();

            // contains the amount of bins for the i-th bar
            let mut ranges = Vec::new();

            let mut start = 1;
            for &weight in weights[..(weights.len() - 1)].iter() {
                let mut end =
                    start + ((weight / weight_sum) * (amount_bins as f32)).round() as usize;

                if end == start {
                    end += 1;
                }
                ranges.push(start..end);
                start = end;
            }

            // add range for the last bar
            ranges.push(start..bin_range.end);

            ranges.into_boxed_slice()
        };

        Self {
            bar_values,
            bar_bin_indices,
            sensitivity: sensitivity.unwrap_or(1.),
        }
    }

    pub fn process(&mut self, fft_out: &[Complex32]) -> &[f32] {
        let mut overshoot = false;
        let mut is_silent = true;
        for (i, range) in self.bar_bin_indices.iter().cloned().enumerate() {
            let range_len = range.len();
            let bar_val = fft_out[range]
                .iter()
                .map(|out| {
                    let mag = out.norm();
                    if mag > 0. {
                        is_silent = false;
                    }
                    mag
                })
                .sum::<f32>()
                / range_len as f32;

            let raw_new_bar_value = {
                let log_factor = ((i + 4) as f32).log(4.);
                let exp_factor = 1.05f32.powf((i + 1) as f32);

                bar_val * self.sensitivity * log_factor * exp_factor
            };

            let prev_bar_value = self.bar_values[i];
            self.bar_values[i] =
                (0.8 * raw_new_bar_value + 0.2 * prev_bar_value).max(prev_bar_value * 0.9);

            if self.bar_values[i] > 1. {
                overshoot = true;
            }
        }

        if overshoot {
            self.sensitivity *= 0.98;
        } else if !is_silent {
            self.sensitivity *= 1.002;
        }

        &self.bar_values
    }

    pub fn sensitivity(&self) -> f32 {
        self.sensitivity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bins_with_less_bars_than_samples() {
        let e = Equalizer::new(3, 0..10, 10, SampleRate(10));

        assert_eq!(e.bar_bin_indices[0], 0..3);
        assert_eq!(e.bar_bin_indices[1], 3..6);
        assert_eq!(e.bar_bin_indices[2], 6..9);
        assert_eq!(e.bar_bin_indices.len(), 3, "{:?}", e.bar_bin_indices);
    }
}
