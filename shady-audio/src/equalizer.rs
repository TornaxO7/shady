use core::f32;
use std::ops::Range;

use cpal::SampleRate;
use realfft::num_complex::Complex32;
use tracing::{debug, instrument};

use crate::{Hz, MAX_HUMAN_FREQUENCY, MIN_HUMAN_FREQUENCY};

const DEFAULT_INIT_SENSITIVITY: f32 = 1.;

#[derive(Debug)]
pub struct Equalizer {
    bar_values: Box<[f32]>,
    bar_ranges: Box<[Range<usize>]>,
    started_falling: Box<[bool]>,

    sensitivity: f32,
}

impl Equalizer {
    #[instrument(name = "Equalizer::new")]
    pub fn new(
        amount_bars: usize,
        freq_range: Range<Hz>,
        sample_len: usize, // = fft size
        sample_rate: SampleRate,
        init_sensitivity: Option<f32>,
    ) -> Self {
        assert!(sample_rate.0 > 0);

        let bar_values = vec![0.; amount_bars].into_boxed_slice();
        let started_falling = vec![false; amount_bars].into_boxed_slice();

        let bar_ranges = {
            let freq_resolution = sample_rate.0 as f32 / sample_len as f32;
            debug!("Freq resolution: {}", freq_resolution);

            let weights = (0..amount_bars)
                .map(|index| exp_fun(index as f32 / amount_bars as f32))
                .collect::<Vec<f32>>();
            debug!("Weights: {:?}", weights);

            // the relevant index range of the fft output which we should use for the bars
            let bin_range = Range {
                start: ((freq_range.start as f32 / freq_resolution) as usize).max(1),
                end: (freq_range.end as f32 / freq_resolution).ceil() as usize,
            };
            let amount_bins = bin_range.len();
            debug!("Bin range: {:?}", bin_range);
            debug!("Available bins: {}", amount_bins);

            assert!(
                amount_bins >= amount_bars,
                "Not enough bins available (available: {}) for {} bars",
                amount_bins,
                amount_bars
            );

            let ranges = {
                let mut cut_offs = Vec::with_capacity(amount_bars);
                let mut start = 0;

                for weight in weights {
                    let mut end = ((weight / MAX_HUMAN_FREQUENCY as f32) * amount_bins as f32)
                        .ceil() as usize;
                    if start >= end {
                        end = start + 1;
                    }

                    cut_offs.push(start..end);
                    start = end;
                }
                // let the last bar use every resulting bar
                let last_range = cut_offs
                    .last_mut()
                    .expect("There's at least one range/bar.");
                last_range.end = amount_bins;

                cut_offs
            };
            tracing::debug!("Bin ranges: {:?}", ranges);

            ranges.into_boxed_slice()
        };

        Self {
            bar_values,
            bar_ranges,
            started_falling,
            sensitivity: init_sensitivity.unwrap_or(DEFAULT_INIT_SENSITIVITY),
        }
    }

    pub fn process(&mut self, fft_out: &[Complex32]) -> &[f32] {
        let mut overshoot = false;
        let mut is_silent = true;

        for (i, range) in self.bar_ranges.iter().cloned().enumerate() {
            let prev_magnitude = self.bar_values[i];
            let next_magnitude: f32 = {
                let raw_bar_val = fft_out[range]
                    .iter()
                    .map(|out| {
                        let mag = out.norm();
                        if mag > 0. {
                            is_silent = false;
                        }
                        mag
                    })
                    .max_by(|a, b| a.total_cmp(b))
                    .unwrap();

                self.sensitivity
                    * raw_bar_val
                    * 10f32.powf((i as f32 / self.bar_values.len() as f32) - 1.1)
            };

            debug_assert!(!prev_magnitude.is_nan());
            debug_assert!(!next_magnitude.is_nan());

            let rel_change = next_magnitude / prev_magnitude;
            if is_silent {
                self.bar_values[i] *= 0.75;
                self.started_falling[i] = false;
            } else {
                let was_already_falling = self.started_falling[i];
                if next_magnitude < prev_magnitude && !was_already_falling {
                    self.started_falling[i] = true;
                    self.bar_values[i] += (next_magnitude - prev_magnitude) * 0.1;
                } else {
                    self.started_falling[i] = false;
                    self.bar_values[i] +=
                        (next_magnitude - prev_magnitude) * rel_change.clamp(0.05, 0.2);
                }
            }

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

fn exp_fun(x: f32) -> f32 {
    debug_assert!(0. <= x);
    debug_assert!(x <= 1.);

    let max_mel_value = mel(MAX_HUMAN_FREQUENCY as f32);
    let min_mel_value = mel(MIN_HUMAN_FREQUENCY as f32);

    // map [0, 1] => [min-mel-value, max-mel-value]
    let mapped_x = x * (max_mel_value - min_mel_value) + min_mel_value;
    inv_mel(mapped_x)
}

// https://en.wikipedia.org/wiki/Mel_scale
fn mel(x: f32) -> f32 {
    debug_assert!(MIN_HUMAN_FREQUENCY as f32 <= x);
    debug_assert!(x <= MAX_HUMAN_FREQUENCY as f32);

    2595. * (1. + x / 700.).log10()
}

fn inv_mel(x: f32) -> f32 {
    let min_mel_value = mel(MIN_HUMAN_FREQUENCY as f32);
    let max_mel_value = mel(MAX_HUMAN_FREQUENCY as f32);

    debug_assert!(min_mel_value <= x);
    debug_assert!(x <= max_mel_value);

    700. * (10f32.powf(x / 2595.) - 1.)
}
