use std::ops::Range;

use cpal::SampleRate;
use realfft::num_complex::Complex32;
use tracing::{debug, instrument};

use crate::Hz;

type HzF32 = f32;

#[derive(Debug, Clone)]
struct BarInfo {
    avg_freq: HzF32,
    bin_range: Range<usize>,
}

#[derive(Debug)]
pub struct Equalizer {
    bar_values: Box<[f32]>,
    bar_infos: Box<[BarInfo]>,

    sensitivity: f32,
}

impl Equalizer {
    #[instrument(name = "Equalizer::new")]
    pub fn new(
        amount_bars: usize,
        freq_range: Range<Hz>,
        sample_len: usize, // = fft size
        sample_rate: SampleRate,
        sensitivity: Option<f32>,
    ) -> Self {
        let bar_values = vec![0.; amount_bars].into_boxed_slice();

        let bar_infos = {
            let freq_resolution = sample_rate.0 as f32 / sample_len as f32;

            // the relevant index range of the fft output which we should use for the bars
            let bin_range = Range {
                start: ((freq_range.start as f32 / freq_resolution) as usize).max(1),
                end: (freq_range.end as f32 / freq_resolution).ceil() as usize,
            };
            let amount_bins = bin_range.len();

            debug_assert!(
                amount_bins >= amount_bars,
                "Not enough bins available (available: {}) for {} bars",
                amount_bins,
                amount_bars
            );

            let mut ranges = Vec::with_capacity(amount_bars);

            let step = amount_bins / amount_bars;
            let remains = amount_bins % amount_bars;
            for start in (bin_range.start..(bin_range.end - remains)).step_by(step) {
                let bar_bin_range = start..(start + step);
                let avg_freq = bar_bin_range
                    .clone()
                    .into_iter()
                    .map(|bin_index| bin_index as f32 * freq_resolution)
                    .sum::<f32>()
                    / step as f32;

                ranges.push(BarInfo {
                    avg_freq,
                    bin_range: bar_bin_range,
                });
            }
            ranges
                .last_mut()
                .map(|bar_info| bar_info.bin_range.end = bin_range.end);

            debug!("Amount bars: {}", amount_bars);
            debug!("Amount available bins: {}", amount_bins);
            debug!("Amount bins per bar: {}", step);
            debug!("Bin ranges: {:?}", ranges);

            ranges.into_boxed_slice()
        };

        Self {
            bar_values,
            bar_infos,
            sensitivity: sensitivity.unwrap_or(1.),
        }
    }

    pub fn process(&mut self, fft_out: &[Complex32]) -> &[f32] {
        let mut overshoot = false;
        let mut is_silent = true;
        for (i, bar_info) in self.bar_infos.iter().cloned().enumerate() {
            let prev_magnitude = self.bar_values[i];
            let next_magnitude = {
                let range_len = bar_info.bin_range.len();
                let raw_bar_val = fft_out[bar_info.bin_range]
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

                // https://en.wikipedia.org/wiki/Mel_scale#History_and_other_formulas
                let mel = 2595. * (1. + bar_info.avg_freq / 700.).log10();

                mel * raw_bar_val * self.sensitivity
            };

            debug_assert!(!prev_magnitude.is_nan());
            debug_assert!(!next_magnitude.is_nan());

            if is_silent {
                self.bar_values[i] = prev_magnitude * 0.9;
            } else {
                let relative_change = if prev_magnitude > 0. {
                    next_magnitude / prev_magnitude
                } else {
                    1.
                };

                let factor = if relative_change >= 2. {
                    0.8
                } else if 0.9 <= relative_change && relative_change <= 1.1 {
                    0.1
                } else {
                    relative_change
                };

                self.bar_values[i] = factor * next_magnitude + (1. - factor) * prev_magnitude;
            }

            if self.bar_values[i] > 1. {
                overshoot = true;
            }
        }

        if overshoot {
            self.sensitivity *= 0.95;
        } else if !is_silent {
            self.sensitivity *= 1.002;
        }

        &self.bar_values
    }

    pub fn sensitivity(&self) -> f32 {
        self.sensitivity
    }
}
