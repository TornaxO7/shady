mod config;

use std::ops::Range;

pub use config::{Config, InterpolationVariant};
use realfft::num_complex::Complex32;
use tracing::debug;

use crate::{
    interpolation::{
        CubicSplineInterpolation, Interpolater, InterpolationInner, LinearInterpolation,
        NothingInterpolation, SupportingPoint,
    },
    SampleProcessor, MAX_HUMAN_FREQUENCY, MIN_HUMAN_FREQUENCY,
};

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

/// The struct which computates the bar values of the samples of the fetcher.
pub struct BarProcessor {
    sensitivity: f32,

    supporting_point_infos: Box<[SupportingPointInfo]>,
    interpolator: Box<dyn Interpolater>,

    config: Config,
}

impl BarProcessor {
    /// Creates a new instance.
    ///
    /// See the examples of this crate to see it's usage.
    pub fn new(processor: &SampleProcessor, config: Config) -> Self {
        let Config {
            interpolation,
            amount_bars,
            freq_range,
        } = config.clone();

        let (supporting_points, supporting_point_infos) = {
            let sample_rate = processor.sample_rate();
            let sample_len = processor.fft_size();

            let mut supporting_points = Vec::new();
            let mut supporting_point_infos = Vec::new();

            // == preparations
            let weights = (0..u16::from(amount_bars))
                .map(|index| exp_fun((index + 1) as f32 / (u16::from(amount_bars) + 1) as f32))
                .collect::<Vec<f32>>();
            debug!("Weights: {:?}", weights);

            let amount_bins = {
                let freq_resolution = sample_rate.0 as f32 / sample_len as f32;
                debug!("Freq resolution: {}", freq_resolution);

                // the relevant index range of the fft output which we should use for the bars
                let bin_range = Range {
                    start: ((u16::from(freq_range.start) as f32 / freq_resolution) as usize).max(1),
                    end: (u16::from(freq_range.end) as f32 / freq_resolution).ceil() as usize,
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

        let interpolator: Box<dyn Interpolater> = match interpolation {
            InterpolationVariant::None => NothingInterpolation::boxed(supporting_points),
            InterpolationVariant::Linear => LinearInterpolation::boxed(supporting_points),
            InterpolationVariant::CubicSpline => CubicSplineInterpolation::boxed(supporting_points),
        };

        Self {
            sensitivity: 1.,
            supporting_point_infos,
            interpolator,
            config,
        }
    }

    /// Returns the values for each bar.
    pub fn process_bars(&mut self, processor: &SampleProcessor) -> &[f32] {
        let (overshoot, is_silent) = self.update_supporting_points(processor.fft_out());
        if overshoot {
            self.sensitivity *= 0.98;
        } else if !is_silent {
            self.sensitivity *= 1.002;
        }

        self.interpolator.interpolate()
    }

    fn update_supporting_points(&mut self, fft_out: &[Complex32]) -> (bool, bool) {
        let mut overshoot = false;
        let mut is_silent = true;

        for (supporting_point, info) in self
            .interpolator
            .supporting_points_mut()
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

                self.sensitivity
                    * raw_bar_val
                    * 10f32.powf((x as f32 / u16::from(self.config.amount_bars) as f32) - 1.1)
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

    /// Returns its config.
    pub fn config(&self) -> &Config {
        &self.config
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
