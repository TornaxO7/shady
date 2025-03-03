use core::f32;
use std::{num::NonZeroUsize, ops::Range};

use cpal::SampleRate;
use realfft::num_complex::Complex32;
use tracing::{debug, instrument};

use crate::{Hz, MAX_HUMAN_FREQUENCY, MIN_HUMAN_FREQUENCY};

const DEFAULT_INIT_SENSITIVITY: f32 = 1.;

#[derive(Debug, Clone)]
struct AnchorSection {
    // the range within the fft output which should be used
    fft_range: Range<usize>,
    // which bar value should get the value
    bar_value_idx: usize,
}

#[derive(Debug, Clone)]
struct InterpolationSection {
    // the starting index within `bar_values` which should be calculated
    start: usize,
    amount: NonZeroUsize,
}

#[derive(Debug)]
pub struct Equalizer {
    bar_values: Box<[f32]>,
    started_falling: Box<[bool]>,

    anchor_sections: Box<[AnchorSection]>,
    interpolation_sections: Box<[InterpolationSection]>,

    sensitivity: f32,
    is_silent: bool,
    overshoot: bool,
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

        let (anchor_sections, interpolation_sections) = {
            // == preparations
            let weights = (1..(amount_bars + 1))
                .map(|index| exp_fun(index as f32 / amount_bars as f32))
                .collect::<Vec<f32>>();
            debug!("Weights: {:?}", weights);

            let amount_bins = {
                let freq_resolution = sample_rate.0 as f32 / sample_len as f32;
                debug!("Freq resolution: {}", freq_resolution);

                // the relevant index range of the fft output which we should use for the bars
                let bin_range = Range {
                    start: ((freq_range.start as f32 / freq_resolution) as usize).max(1),
                    end: (freq_range.end as f32 / freq_resolution).ceil() as usize,
                };
                debug!("Bin range: {:?}", bin_range);
                bin_range.len()
            };
            debug!("Available bins: {}", amount_bins);

            // == fill sections
            let mut anchor_sections = Vec::new();
            let mut interpolation_sections = Vec::new();

            let mut interpol_section: Option<InterpolationSection> = None;
            let mut prev_fft_range = 0..0;

            for (bar_value_idx, weight) in weights.iter().enumerate() {
                let end =
                    ((weight / MAX_HUMAN_FREQUENCY as f32) * amount_bins as f32).ceil() as usize;

                let new_fft_range = prev_fft_range.end..end;
                let is_interpolation_section =
                    new_fft_range == prev_fft_range || new_fft_range.is_empty();
                if is_interpolation_section {
                    // interpolate
                    if let Some(inter) = interpol_section.as_mut() {
                        inter.amount = inter.amount.saturating_add(1);
                    } else {
                        interpol_section = Some(InterpolationSection {
                            start: bar_value_idx,
                            amount: NonZeroUsize::new(1).unwrap(),
                        });
                    }
                } else {
                    // new anchor
                    if let Some(inter) = interpol_section.clone() {
                        interpolation_sections.push(inter);
                        interpol_section = None;
                    }

                    anchor_sections.push(AnchorSection {
                        fft_range: new_fft_range.clone(),
                        bar_value_idx,
                    });
                }

                prev_fft_range = new_fft_range;
            }

            assert!(interpol_section.is_none());

            (
                anchor_sections.into_boxed_slice(),
                interpolation_sections.into_boxed_slice(),
            )
        };

        debug!("Anchor sections: {:#?}", &anchor_sections);
        debug!("Interpolation sections: {:#?}", &interpolation_sections);

        Self {
            bar_values,
            anchor_sections,
            interpolation_sections,
            started_falling,
            sensitivity: init_sensitivity.unwrap_or(DEFAULT_INIT_SENSITIVITY),
            overshoot: false,
            is_silent: true,
        }
    }

    pub fn process(&mut self, fft_out: &[Complex32]) -> &[f32] {
        self.overshoot = false;
        self.is_silent = true;

        self.process_anchors(fft_out);
        self.process_interpolate();

        if self.overshoot {
            self.sensitivity *= 0.98;
        } else if !self.is_silent {
            self.sensitivity *= 1.002;
        }

        &self.bar_values
    }

    pub fn sensitivity(&self) -> f32 {
        self.sensitivity
    }

    fn process_anchors(&mut self, fft_out: &[Complex32]) {
        for section in self.anchor_sections.iter() {
            let i = section.bar_value_idx;

            let prev_magnitude = self.bar_values[i];
            let next_magnitude = {
                let raw_bar_val = fft_out[section.fft_range.clone()]
                    .iter()
                    .map(|out| {
                        let mag = out.norm();
                        if mag > 0. {
                            self.is_silent = false;
                        }

                        mag
                    })
                    .max_by(|a, b| a.total_cmp(b))
                    .unwrap();

                self.sensitivity
                    * raw_bar_val
                    * 10f32
                        .powf((section.bar_value_idx as f32 / self.bar_values.len() as f32) - 1.1)
            };

            debug_assert!(!prev_magnitude.is_nan());
            debug_assert!(!next_magnitude.is_nan());

            let rel_change = next_magnitude / prev_magnitude;
            if self.is_silent {
                self.bar_values[i] *= 0.75;
                self.started_falling[i] = false;
            } else {
                let was_falling_before = self.started_falling[i];
                let is_falling = next_magnitude < prev_magnitude;

                if is_falling && !was_falling_before {
                    self.started_falling[i] = true;
                    self.bar_values[i] += (next_magnitude - prev_magnitude) * 0.1;
                } else {
                    self.started_falling[i] = false;
                    self.bar_values[i] +=
                        (next_magnitude - prev_magnitude) * rel_change.clamp(0.05, 0.2);
                }
            }

            if self.bar_values[i] > 1. {
                self.overshoot = true;
            }
        }
    }

    fn process_interpolate(&mut self) {
        for section in self.interpolation_sections.iter() {
            let amount = usize::from(section.amount);
            let start_anchor_value = self.bar_values[section.start - 1];
            let end_anchor_value = self.bar_values[section.start + amount];

            let range = section.start..(section.start + amount);
            for (i, bar_value_idx) in range.enumerate() {
                let t = (i + 1) as f32 / (amount + 1) as f32;
                self.bar_values[bar_value_idx] =
                    t * start_anchor_value + (1. - t) * end_anchor_value;
            }
        }
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
