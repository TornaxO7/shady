use core::f32;
use std::{fmt::Write, num::NonZeroUsize, ops::Range};

use cpal::SampleRate;
use nalgebra::{DMatrix, DVector};
use realfft::num_complex::Complex32;
use tracing::{debug, instrument};

use crate::{Hz, MAX_HUMAN_FREQUENCY, MIN_HUMAN_FREQUENCY};

const DEFAULT_INIT_SENSITIVITY: f32 = 1.;

#[derive(Debug, Clone)]
struct SupportingPoint {
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

    // the index within the `supporting_points` vector of the ending supporting point
    // within the section
    ending_supporting_point_idx: usize,
}

#[derive(Debug)]
pub struct Equalizer {
    bar_values: Box<[f32]>,
    started_falling: Box<[bool]>,

    supporting_points: Box<[SupportingPoint]>,
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

        let (supporting_points, isections) = {
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
            let mut supporting_points = Vec::new();
            let mut isections = Vec::new();

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
                            ending_supporting_point_idx: 0,
                        });
                    }
                } else {
                    // new anchor
                    if let Some(mut inter) = interpol_section.clone() {
                        inter.ending_supporting_point_idx = supporting_points.len();
                        isections.push(inter);
                        interpol_section = None;
                    }

                    supporting_points.push(SupportingPoint {
                        fft_range: new_fft_range.clone(),
                        bar_value_idx,
                    });
                }

                prev_fft_range = new_fft_range;
            }

            assert!(interpol_section.is_none());

            (
                supporting_points.into_boxed_slice(),
                isections.into_boxed_slice(),
            )
        };

        debug!("Anchor sections: {:#?}", &supporting_points);
        debug!("Interpolation sections: {:#?}", &isections);

        Self {
            bar_values,
            supporting_points,
            interpolation_sections: isections,
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
        for section in self.supporting_points.iter() {
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
        self.process_cubic_spline_interpolation();
        // self.process_linear_interpolation();
    }

    fn process_cubic_spline_interpolation(&mut self) {
        let section_widths = {
            let mut section_width = Vec::with_capacity(self.supporting_points.len() - 1);

            let mut prev_width = self.supporting_points[0].bar_value_idx;
            for anchor in self.supporting_points[1..].iter() {
                let width = anchor.bar_value_idx - prev_width;
                prev_width = anchor.bar_value_idx;

                assert!(width > 0);

                section_width.push(width);
            }

            section_width
        };
        let amount_sections = section_widths.len();
        debug_assert!(amount_sections + 1 == self.supporting_points.len());
        debug!("Section widths:\n{:?}", section_widths);

        let matrix = {
            let mut matrix = DMatrix::zeros(amount_sections, amount_sections);

            // add first row
            {
                let mut first_row = matrix.row_mut(0);
                first_row[0] = 2. * section_widths[0] as f32;
                first_row[1] = section_widths[0] as f32;
            }

            // add rows in between
            {
                let mut offset = 0;
                for row_idx in 1..(amount_sections - 1) {
                    let mut row = matrix.row_mut(row_idx);

                    row[offset] = section_widths[offset] as f32;
                    row[offset + 1] =
                        2. * (section_widths[offset] + section_widths[offset + 1]) as f32;
                    row[offset + 2] = section_widths[offset + 1] as f32;

                    offset += 1;
                }
            }

            // add last row
            {
                let mut last_row = matrix.row_mut(amount_sections - 1);

                last_row[amount_sections - 2] = section_widths[amount_sections - 1] as f32;
                last_row[amount_sections - 1] = 2. * section_widths[amount_sections - 1] as f32;
            }

            // just for debugging purposes
            {
                let mut dbg_matrix_msg = String::new();
                for row_idx in 0..amount_sections {
                    let row = matrix.row(row_idx);
                    write!(&mut dbg_matrix_msg, "[").unwrap();
                    for &value in row.columns_range(..amount_sections - 1) {
                        write!(&mut dbg_matrix_msg, "{value}, ").unwrap();
                    }
                    write!(&mut dbg_matrix_msg, "{}]\n", row[amount_sections - 1]).unwrap();
                }
                debug!("Matrix:\n{}", dbg_matrix_msg)
            }

            matrix * (1. / 6.)
        };
        debug_assert_eq!(matrix.row(0).len(), amount_sections);
        debug_assert_eq!(matrix.column(0).len(), amount_sections);

        let gradients = {
            let mut gradients = Vec::with_capacity(amount_sections);

            for (i, right) in self.supporting_points[1..].iter().enumerate() {
                let left = &self.supporting_points[i];
                let left_x = left.bar_value_idx;
                let left_y = self.bar_values[left_x];

                let right_x = right.bar_value_idx;
                let right_y = self.bar_values[right_x];

                let gradient = (left_y - right_y) / (left_x as f32 - right_x as f32);

                gradients.push(gradient);
            }

            gradients
        };
        debug_assert_eq!(gradients.len(), amount_sections);
        debug!("Gradients: {:?}", gradients);

        let gradient_diffs = {
            let mut gradient_diffs = Vec::with_capacity(amount_sections);

            // first diff
            gradient_diffs.push(gradients[0]);

            // d1 to d(N - 1)
            for (i, next_gradient) in gradients[1..gradients.len() - 1]
                .iter()
                .cloned()
                .enumerate()
            {
                let prev_gradient = gradients[i];

                gradient_diffs.push(next_gradient - prev_gradient);
            }

            // last diff
            gradient_diffs.push(gradients[gradients.len() - 1]);

            gradient_diffs
        };
        debug_assert_eq!(gradient_diffs.len(), amount_sections);

        let l = matrix
            .cholesky()
            .expect("Hold up! Looks like I failed my numeric exam... ;------;");
        let gammas = l.solve(&DVector::from_row_slice(gradient_diffs.as_slice()));
        debug_assert_eq!(gammas.len(), amount_sections);

        // now let's do the actual calculation
        for section in self.interpolation_sections.iter() {
            let n = section.ending_supporting_point_idx;

            let prev_supporting_point = &self.supporting_points[n - 1];
            let prev_x = prev_supporting_point.bar_value_idx;
            let prev_y = self.bar_values[prev_x];
            let prev_gamma = gammas[n - 1];

            let next_supporting_point = &self.supporting_points[n];
            let next_x = next_supporting_point.bar_value_idx;
            let next_gamma = gammas[n];

            let gradient = gradients[n - 1];
            let section_width = section_widths[n - 1];

            let amount = usize::from(section.amount);
            for idx in 0..amount {
                let idx = usize::from(idx);

                let x = (idx + 1 + prev_x) as f32;

                let interpolated_value = prev_y
                    + (x - prev_x as f32) * gradient
                    + ((x - prev_x as f32) * (x - next_x as f32)) / (6. * section_width as f32)
                        * ((prev_gamma + 2. * next_gamma) * (x - prev_x as f32)
                            - (2. * prev_gamma + next_gamma) * (x - next_x as f32));

                self.bar_values[section.start + idx] = interpolated_value;
                debug!("Interpolated values: {}", interpolated_value);
            }
        }
    }

    #[allow(dead_code)]
    fn process_linear_interpolation(&mut self) {
        for section in self.interpolation_sections.iter() {
            let amount = usize::from(section.amount);
            let start_anchor_value = self.bar_values[section.start - 1];
            let end_anchor_value = self.bar_values[section.start + amount];

            let range = section.start..(section.start + amount);
            for (i, bar_value_idx) in range.enumerate() {
                let t = (i + 1) as f32 / (amount + 1) as f32;
                self.bar_values[bar_value_idx] =
                    t * end_anchor_value + (1. - t) * start_anchor_value;
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
