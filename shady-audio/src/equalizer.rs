use core::f32;
use std::ops::Range;

use cpal::SampleRate;
use realfft::num_complex::Complex32;
use tracing::{debug, instrument};

use crate::{
    interpolation::{
        Interpolater, InterpolationInstantiator, LinearInterpolation, SupportingPoint,
    },
    Hz, MAX_HUMAN_FREQUENCY, MIN_HUMAN_FREQUENCY,
};

const DEFAULT_INIT_SENSITIVITY: f32 = 1.;

/// Additional information about the supporting points
#[derive(Debug)]
struct SupportingPointInfo {
    /// which fft bins of the fft output should be used for the given bar
    fft_bin_range: Range<usize>,
    /// if the bar just started falling
    started_falling: bool,
}

impl SupportingPointInfo {
    pub fn new(fft_bin_range: Range<usize>) -> Self {
        Self {
            fft_bin_range,
            started_falling: false,
        }
    }
}

pub struct Equalizer {
    sensitivity: f32,

    supporting_point_infos: Box<[SupportingPointInfo]>,
    interpolator: Box<dyn Interpolater>,
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

        let (supporting_points, supporting_point_infos) = {
            let mut supporting_points = Vec::new();
            let mut supporting_point_infos = Vec::with_capacity(amount_bars);

            // == preparations
            let weights = (0..amount_bars)
                .map(|index| exp_fun((index + 1) as f32 / (amount_bars + 1) as f32))
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

            // == calculate sections
            let mut prev_fft_range = 0..0;
            for (bar_idx, weight) in weights.iter().enumerate() {
                let end =
                    ((weight / MAX_HUMAN_FREQUENCY as f32) * amount_bins as f32).ceil() as usize;

                let new_fft_range = prev_fft_range.end..end;
                let is_supporting_point =
                    new_fft_range != prev_fft_range && !new_fft_range.is_empty();
                if is_supporting_point {
                    supporting_points.push(SupportingPoint { x: bar_idx, y: 0. });

                    supporting_point_infos.push(SupportingPointInfo::new(new_fft_range.clone()));
                }

                prev_fft_range = new_fft_range;
            }

            (supporting_points, supporting_point_infos.into_boxed_slice())
        };

        let interpolator = LinearInterpolation::boxed(supporting_points);

        Self {
            supporting_point_infos,
            sensitivity: init_sensitivity.unwrap_or(DEFAULT_INIT_SENSITIVITY),
            interpolator,
        }
    }

    pub fn process(&mut self, fft_out: &[Complex32]) -> &[f32] {
        let (overshoot, is_silent) = self.update_supporting_points(fft_out);
        if overshoot {
            self.sensitivity *= 0.98;
        } else if !is_silent {
            self.sensitivity *= 1.002;
        }

        self.interpolator.interpolate()
    }

    pub fn sensitivity(&self) -> f32 {
        self.sensitivity
    }

    fn update_supporting_points(&mut self, fft_out: &[Complex32]) -> (bool, bool) {
        let mut overshoot = false;
        let mut is_silent = true;

        let amount_bars = self.interpolator.total_amount_entries();

        for (supporting_point, info) in self
            .interpolator
            .supporting_points_mut()
            .into_iter()
            .zip(self.supporting_point_infos.iter_mut())
        {
            let x = supporting_point.x;
            let prev_magnitude = supporting_point.y;
            let next_magnitude = {
                let raw_bar_val = fft_out[info.fft_bin_range.clone()]
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

                self.sensitivity * raw_bar_val * 10f32.powf((x as f32 / amount_bars as f32) - 1.1)
            };

            debug_assert!(!prev_magnitude.is_nan());
            debug_assert!(!next_magnitude.is_nan());

            let rel_change = next_magnitude / prev_magnitude;
            if is_silent {
                supporting_point.y *= 0.75;
                info.started_falling = false;
            } else {
                let was_falling_before = info.started_falling;
                let is_falling = next_magnitude < prev_magnitude;

                if is_falling && !was_falling_before {
                    info.started_falling = true;
                    supporting_point.y += (next_magnitude - prev_magnitude) * 0.1;
                } else {
                    info.started_falling = false;
                    supporting_point.y +=
                        (next_magnitude - prev_magnitude) * rel_change.clamp(0.05, 0.2);
                }
            }

            if supporting_point.y > 1. {
                overshoot = true;
            }
        }

        (overshoot, is_silent)
    }

    // fn process_cubic_spline_interpolation(&mut self) {
    //     let section_widths = {
    //         let mut section_width = Vec::with_capacity(self.supporting_points.len() - 1);

    //         let mut prev_width = self.supporting_points[0].x;
    //         for anchor in self.supporting_points[1..].iter() {
    //             let width = anchor.x - prev_width;
    //             prev_width = anchor.x;

    //             assert!(width > 0);

    //             section_width.push(width);
    //         }

    //         section_width
    //     };
    //     let amount_sections = section_widths.len();
    //     debug_assert!(amount_sections + 1 == self.supporting_points.len());
    //     debug!("Section widths:\n{:?}", section_widths);

    //     let matrix = {
    //         let mut matrix = DMatrix::zeros(amount_sections, amount_sections);

    //         // add first row
    //         {
    //             let mut first_row = matrix.row_mut(0);
    //             first_row[0] = 2. * section_widths[0] as f32;
    //             first_row[1] = section_widths[0] as f32;
    //         }

    //         // add rows in between
    //         {
    //             let mut offset = 0;
    //             for row_idx in 1..(amount_sections - 1) {
    //                 let mut row = matrix.row_mut(row_idx);

    //                 row[offset] = section_widths[offset] as f32;
    //                 row[offset + 1] =
    //                     2. * (section_widths[offset] + section_widths[offset + 1]) as f32;
    //                 row[offset + 2] = section_widths[offset + 1] as f32;

    //                 offset += 1;
    //             }
    //         }

    //         // add last row
    //         {
    //             let mut last_row = matrix.row_mut(amount_sections - 1);

    //             last_row[amount_sections - 2] = section_widths[amount_sections - 1] as f32;
    //             last_row[amount_sections - 1] = 2. * section_widths[amount_sections - 1] as f32;
    //         }

    //         // just for debugging purposes
    //         {
    //             let mut dbg_matrix_msg = String::new();
    //             for row_idx in 0..amount_sections {
    //                 let row = matrix.row(row_idx);
    //                 write!(&mut dbg_matrix_msg, "[").unwrap();
    //                 for &value in row.columns_range(..amount_sections - 1) {
    //                     write!(&mut dbg_matrix_msg, "{value}, ").unwrap();
    //                 }
    //                 write!(&mut dbg_matrix_msg, "{}]\n", row[amount_sections - 1]).unwrap();
    //             }
    //             debug!("Matrix:\n{}", dbg_matrix_msg)
    //         }

    //         matrix * (1. / 6.)
    //     };
    //     debug_assert_eq!(matrix.row(0).len(), amount_sections);
    //     debug_assert_eq!(matrix.column(0).len(), amount_sections);

    //     let gradients = {
    //         let mut gradients = Vec::with_capacity(amount_sections);

    //         for (i, right) in self.supporting_points[1..].iter().enumerate() {
    //             let left = &self.supporting_points[i];
    //             let left_x = left.x;
    //             let left_y = self.bar_values[left_x];

    //             let right_x = right.x;
    //             let right_y = self.bar_values[right_x];

    //             let gradient = (left_y - right_y) / (left_x as f32 - right_x as f32);

    //             gradients.push(gradient);
    //         }

    //         gradients
    //     };
    //     debug_assert_eq!(gradients.len(), amount_sections);
    //     debug!("Gradients: {:?}", gradients);

    //     let gradient_diffs = {
    //         let mut gradient_diffs = Vec::with_capacity(amount_sections);

    //         // first diff
    //         gradient_diffs.push(gradients[0]);

    //         // d1 to d(N - 1)
    //         for (i, next_gradient) in gradients[1..gradients.len() - 1]
    //             .iter()
    //             .cloned()
    //             .enumerate()
    //         {
    //             let prev_gradient = gradients[i];

    //             gradient_diffs.push(next_gradient - prev_gradient);
    //         }

    //         // last diff
    //         gradient_diffs.push(gradients[gradients.len() - 1]);

    //         gradient_diffs
    //     };
    //     debug_assert_eq!(gradient_diffs.len(), amount_sections);

    //     let l = matrix
    //         .cholesky()
    //         .expect("Hold up! Looks like I failed my numeric exam... ;------;");
    //     let gammas = l.solve(&DVector::from_row_slice(gradient_diffs.as_slice()));
    //     debug_assert_eq!(gammas.len(), amount_sections);

    //     // now let's do the actual calculation
    //     for section in self.interpolation_sections.iter() {
    //         let n = section.ending_supporting_point_idx;

    //         let prev_supporting_point = &self.supporting_points[n - 1];
    //         let prev_x = prev_supporting_point.x;
    //         let prev_y = self.bar_values[prev_x];
    //         let prev_gamma = gammas[n - 1];

    //         let next_supporting_point = &self.supporting_points[n];
    //         let next_x = next_supporting_point.x;
    //         let next_gamma = gammas[n];

    //         let gradient = gradients[n - 1];
    //         let section_width = section_widths[n - 1];

    //         let amount = usize::from(section.amount);
    //         for idx in 0..amount {
    //             let idx = usize::from(idx);

    //             let x = (idx + 1 + prev_x) as f32;

    //             let interpolated_value = prev_y
    //                 + (x - prev_x as f32) * gradient
    //                 + ((x - prev_x as f32) * (x - next_x as f32)) / (6. * section_width as f32)
    //                     * ((prev_gamma + 2. * next_gamma) * (x - prev_x as f32)
    //                         - (2. * prev_gamma + next_gamma) * (x - next_x as f32));

    //             self.bar_values[section.start + idx] = interpolated_value;
    //             debug!("Interpolated values: {}", interpolated_value);
    //         }
    //     }
    // }

    // }
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
