use std::ops::Range;

use cpal::SampleRate;
use realfft::num_complex::Complex32;

use crate::Hz;

#[derive(Debug)]
pub struct Equalizer {
    amount_bars: usize,
    sample_len: usize,
    bar_values: Box<[f32]>,
    bar_bin_indices: Box<[Range<usize>]>,
}

impl Equalizer {
    pub fn new(
        amount_bars: usize,
        freq_range: Range<Hz>,
        sample_len: usize,
        sample_rate: SampleRate,
    ) -> Self {
        let bar_values = vec![0.; amount_bars].into_boxed_slice();

        let bar_bin_indices = {
            let freq_resolution = sample_rate.0 as f32 / sample_len as f32;

            // the relevant index range of the fft output which we should use for the bars
            let mut bin_range = Range {
                start: ((freq_range.start as f32 / freq_resolution) as usize).max(1),
                end: (freq_range.end as f32 / freq_resolution) as usize,
            };

            let amount_bins_per_bar = bin_range.len() / amount_bars;
            // make the range a multiple of the amount of bins we need per bar for nice computation
            bin_range.end = bin_range.start + amount_bins_per_bar * amount_bars;
            assert!(
                amount_bins_per_bar > 0,
                "One bar needs {} bins but we only have {} bins (for {} bars).",
                amount_bins_per_bar,
                bin_range.len(),
                amount_bars
            );

            let mut indices = Vec::new();

            let bin_range_len = bin_range.len();
            for index in bin_range.step_by(amount_bins_per_bar) {
                let end = (index + amount_bins_per_bar).min(bin_range_len);
                indices.push(index..end);
            }

            indices.into_boxed_slice()
        };

        Self {
            amount_bars,
            sample_len,
            bar_values,
            bar_bin_indices,
        }
    }

    pub fn process(&mut self, fft_out: &[Complex32]) -> &[f32] {
        // let mut max_bar_value = f32::MIN;

        for (i, range) in self.bar_bin_indices.iter().cloned().enumerate() {
            let range_len = range.len() as f32;

            let bar_val = fft_out[range].iter().map(|out| out.norm()).sum::<f32>() / range_len;

            // if bar_val > max_bar_value {
            //     max_bar_value = bar_val;
            // }

            self.bar_values[i] = (bar_val / self.sample_len as f32) * 8. * 1.5;
        }

        // for val in self.bar_values.iter_mut() {
        //     *val /= max_bar_value;
        // }

        &self.bar_values
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
