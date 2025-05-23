mod config;

use std::{num::NonZero, ops::Range};

use config::BarDistribution;
pub use config::{BarProcessorConfig, InterpolationVariant};
use cpal::SampleRate;
use easing_function::{Easing, EasingFunction};
use realfft::num_complex::Complex32;
use tracing::debug;

use crate::{
    interpolation::{
        CubicSplineInterpolation, Interpolater, InterpolationInner, LinearInterpolation,
        NothingInterpolation, SupportingPoint,
    },
    SampleProcessor, MAX_HUMAN_FREQUENCY, MIN_HUMAN_FREQUENCY,
};

/// The struct which computates the bar values of the samples of the fetcher.
pub struct BarProcessor {
    normalize_factor: f32,

    supporting_point_fft_ranges: Box<[Range<usize>]>,
    interpolator: Box<dyn Interpolater>,

    easer: EasingFunction,
    config: BarProcessorConfig,
    sample_rate: SampleRate,
    sample_len: usize,
}

impl BarProcessor {
    /// Creates a new instance.
    ///
    /// See the examples of this crate to see it's usage.
    pub fn new(processor: &SampleProcessor, config: BarProcessorConfig) -> Self {
        let sample_rate = processor.sample_rate();
        let sample_len = processor.fft_size();

        let (interpolator, supporting_point_fft_ranges) =
            Self::new_inner(&config, sample_rate, sample_len);

        Self {
            normalize_factor: 1.,
            supporting_point_fft_ranges,
            interpolator,

            easer: EasingFunction::from(config.easer),
            config,
            sample_rate,
            sample_len,
        }
    }

    /// Returns the values for each bar.
    pub fn process_bars(&mut self, processor: &SampleProcessor) -> &[f32] {
        let (overshoot, is_silent) = self.update_supporting_points(processor.fft_out());
        if overshoot {
            self.normalize_factor *= 0.98;
        } else if !is_silent {
            self.normalize_factor *= 1.002;
        }

        self.interpolator.interpolate()
    }

    pub fn config(&self) -> &BarProcessorConfig {
        &self.config
    }

    fn update_supporting_points(&mut self, fft_out: &[Complex32]) -> (bool, bool) {
        let mut overshoot = false;
        let mut is_silent = true;

        for (supporting_point, fft_range) in self
            .interpolator
            .supporting_points_mut()
            .zip(self.supporting_point_fft_ranges.iter_mut())
        {
            let x = supporting_point.x;
            let prev_magnitude = supporting_point.y;
            let next_magnitude = {
                let mut raw_bar_val = fft_out[fft_range.clone()]
                    .iter()
                    .map(|out| {
                        let mag = out.norm_sqr();
                        if mag > 0. {
                            is_silent = false;
                        }
                        mag
                    })
                    .max_by(|a, b| a.total_cmp(b))
                    .unwrap();

                raw_bar_val = raw_bar_val.sqrt();

                self.normalize_factor
                    * raw_bar_val
                    * 10f32.powf((x as f32 / u16::from(self.config.amount_bars) as f32) - 1.1)
            };

            debug_assert!(!prev_magnitude.is_nan());
            debug_assert!(!next_magnitude.is_nan());

            if is_silent {
                supporting_point.y *= 0.75;
            } else {
                let diff = next_magnitude - prev_magnitude;
                supporting_point.y +=
                    diff * self.easer.ease(diff.abs().min(1.0)) * self.config.sensitivity;
            }

            if supporting_point.y > 1. {
                overshoot = true;
            }
        }

        (overshoot, is_silent)
    }

    /// Change the amount of bars which should be returned.
    ///
    /// # Example
    /// ```rust
    /// use shady_audio::{SampleProcessor, BarProcessor, BarProcessorConfig, fetcher::DummyFetcher};
    ///
    /// let mut sample_processor = SampleProcessor::new(DummyFetcher::new());
    /// let mut bar_processor = BarProcessor::new(
    ///     &sample_processor,
    ///     BarProcessorConfig {
    ///         amount_bars: std::num::NonZero::new(10).unwrap(),
    ///         ..Default::default()
    ///     }
    /// );
    /// sample_processor.process_next_samples();
    ///
    /// let bars = bar_processor.process_bars(&sample_processor);
    /// assert_eq!(bars.len(), 10);
    ///
    /// // change the amount of bars
    /// bar_processor.set_amount_bars(std::num::NonZero::new(20).unwrap());
    /// let bars = bar_processor.process_bars(&sample_processor);
    /// assert_eq!(bars.len(), 20);
    /// ```
    pub fn set_amount_bars(&mut self, amount_bars: NonZero<u16>) {
        self.config.amount_bars = amount_bars;

        let (interpolator, supporting_point_fft_ranges) =
            Self::new_inner(&self.config, self.sample_rate, self.sample_len);

        self.supporting_point_fft_ranges = supporting_point_fft_ranges;
        self.interpolator = interpolator;
    }

    /// Calculates the indexes for the fft output on how to distribute them to each bar.
    fn new_inner(
        config: &BarProcessorConfig,
        sample_rate: SampleRate,
        sample_len: usize,
    ) -> (Box<dyn Interpolater>, Box<[Range<usize>]>) {
        // == preparations
        let weights = (0..u16::from(config.amount_bars))
            .map(|index| exp_fun((index + 1) as f32 / (u16::from(config.amount_bars) + 1) as f32))
            .collect::<Vec<f32>>();
        debug!("Weights: {:?}", weights);

        let amount_bins = {
            let freq_resolution = sample_rate.0 as f32 / sample_len as f32;
            debug!("Freq resolution: {}", freq_resolution);

            // the relevant index range of the fft output which we should use for the bars
            let bin_range = Range {
                start: ((u16::from(config.freq_range.start) as f32 / freq_resolution) as usize)
                    .max(1),
                end: (u16::from(config.freq_range.end) as f32 / freq_resolution).ceil() as usize,
            };
            debug!("Bin range: {:?}", bin_range);
            bin_range.len()
        };
        debug!("Available bins: {}", amount_bins);

        // == supporting points
        let (supporting_points, supporting_point_fft_ranges) = {
            let mut supporting_points = Vec::new();
            let mut supporting_point_fft_ranges = Vec::new();

            let mut prev_fft_range = 0..0;
            for (bar_idx, weight) in weights.iter().enumerate() {
                let end =
                    ((weight / MAX_HUMAN_FREQUENCY as f32) * amount_bins as f32).ceil() as usize;

                let new_fft_range = prev_fft_range.end..end;
                let is_supporting_point =
                    new_fft_range != prev_fft_range && !new_fft_range.is_empty();
                if is_supporting_point {
                    supporting_points.push(SupportingPoint { x: bar_idx, y: 0. });

                    supporting_point_fft_ranges.push(new_fft_range.clone());
                }

                prev_fft_range = new_fft_range;
            }

            // re-adjust the supporting points if needed
            match config.bar_distribution {
                BarDistribution::Uniform => {
                    let step =
                        u16::from(config.amount_bars) as f32 / supporting_points.len() as f32;
                    let supporting_points_len = supporting_points.len();
                    for (idx, supporting_point) in supporting_points
                        [..supporting_points_len.saturating_sub(1)]
                        .iter_mut()
                        .enumerate()
                    {
                        supporting_point.x = (idx as f32 * step) as usize;
                    }
                }
                BarDistribution::Natural => {}
            }

            (supporting_points, supporting_point_fft_ranges)
        };

        // create the interpolator
        let interpolator: Box<dyn Interpolater> = match config.interpolation {
            InterpolationVariant::None => NothingInterpolation::boxed(supporting_points),
            InterpolationVariant::Linear => LinearInterpolation::boxed(supporting_points),
            InterpolationVariant::CubicSpline => CubicSplineInterpolation::boxed(supporting_points),
        };

        (interpolator, supporting_point_fft_ranges.into_boxed_slice())
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
