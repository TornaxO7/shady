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
        sample_len: usize, // = fft size
        sample_rate: SampleRate,
    ) -> Self {
        let bar_values = vec![0.; amount_bars].into_boxed_slice();

        let bar_bin_indices = {
            let freq_resolution = sample_rate.0 as f32 / sample_len as f32;

            // the relevant index range of the fft output which we should use for the bars
            let bin_range = Range {
                start: ((freq_range.start as f32 / freq_resolution) as usize).max(1),
                end: (freq_range.end as f32 / freq_resolution).ceil() as usize,
            };

            tracing::debug!("Relevant bin range: {:?}", bin_range);
            let amount_bins = bin_range.len();

            let weights = {
                let mut weights = Vec::new();

                for i in 0..amount_bars {
                    let weight = (i as f32 + 2.).log2().powf(2.);
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

            tracing::debug!("Ranges({}): {:?}", ranges.len(), ranges);

            ranges.into_boxed_slice()
        };

        Self {
            amount_bars,
            sample_len,
            bar_values,
            bar_bin_indices,
        }
    }

    pub fn process(&mut self, fft_out: &[Complex32]) -> &[f32] {
        for (i, range) in self.bar_bin_indices.iter().cloned().enumerate() {
            // let bar_val = fft_out[range].iter().map(|out| out.norm()).sum::<f32>() / range_len;
            let bar_val = fft_out[range]
                .iter()
                .map(|out| out.norm())
                .max_by(|a, b| a.total_cmp(b))
                .unwrap();

            self.bar_values[i] = bar_val;
        }

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
