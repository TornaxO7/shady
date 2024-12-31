use std::ops::Range;

use crate::{fft, END_FREQ, START_FREQ};
use splines::{Key, Spline};

const EXP_BASE: f32 = 1.06;

/// The spline which represents the audio with the frequency domain.
///
/// It's a wrapper around [splines::Spline] and it's defined as (with mathematical notation of a function):
///
/// ```skip
/// FreqSpline: [0, 1] -> [0, 1]
/// ````
///
/// So the whole input (which is the frequency in this case) range is mapped to `[0, 1]` as well as the output
/// (which is the "presence" of the frequency).
///
/// # Usage
/// You mostly want to use [FreqSpline::sample] or [FreqSpline::clamp_sample].
pub struct FreqSpline {
    spline: Spline<f32, f32>,
    ranges: Vec<Range<usize>>,
}

impl FreqSpline {
    pub(crate) fn new() -> Self {
        let ranges = {
            let mut ranges = Vec::new();

            let mut offset = 1;

            loop {
                let prev = (START_FREQ as f32 * EXP_BASE.powi(offset - 1)) as usize;
                let next = (prev as f32 * EXP_BASE) as usize;
                let next_next = (prev as f32 * EXP_BASE * EXP_BASE) as usize;

                // if the second next range can't use its full range => use everything up
                if next_next > END_FREQ {
                    ranges.push(Range {
                        start: prev,
                        end: fft::FFT_OUTPUT_SIZE,
                    });
                    break;
                } else {
                    ranges.push(Range {
                        start: prev,
                        end: next,
                    });
                };
                offset += 1;
            }

            ranges
        };

        let amount_points = ranges.len();

        // create the spline with equidistant points
        let spline = {
            let mut spline = Spline::from_vec(Vec::with_capacity(amount_points));

            // `-1` in order to for the last point to be at x = 1.
            let step = 1. / (amount_points - 1) as f32;
            for i in 0..amount_points as usize {
                let x = i as f32 * step;
                let key = Key::new(x, 0.0, splines::Interpolation::Linear);
                spline.add(key);
            }
            spline
        };

        #[cfg(debug_assertions)]
        {
            let keys = spline.keys();

            check_t_range(keys);
            check_equidistance(keys);
            check_1_0_point_exists(keys);
        }

        Self { spline, ranges }
    }

    /// Updates the keys according to the magniudes.
    pub(crate) fn update(&mut self, magnitudes: &[f32]) {
        debug_assert_eq!(magnitudes.len(), fft::FFT_OUTPUT_SIZE,
            concat![
                "Current implementation assumes that magnitudes equals the length of the fft output size.\n",
                "One reason is, that we precompute the indexes (see self.ranges) which might be wrong if the magnitude length is different"
            ]
        );

        for (i, range) in self.ranges.iter().enumerate() {
            let value = magnitudes[range.clone()]
                .iter()
                .fold(f32::MIN, |a, &b| a.max(b));
            *self.spline.get_mut(i).unwrap().value = value.min(1.0);
        }
    }

    /// Same as [splines::Spline::sample] but with the condition `0.0 <= t <= 1.0`.
    /// Output is within the range `[0, 1]`.
    pub fn sample(&self, t: f32) -> Option<f32> {
        self.spline.sample(t)
    }

    /// Same as [splines::Spline::clamp_sample].
    /// Output is within the range `[0, 1]`.
    pub fn clamp_sample(&self, t: f32) -> Option<f32> {
        self.spline.clamped_sample(t)
    }
}

#[cfg(debug_assertions)]
fn check_t_range(keys: &[Key<f32, f32>]) {
    for (i, key) in keys.iter().enumerate() {
        assert!(
            0.0 <= key.t && key.t <= 1.0,
            "t value of key (key at index {}) is not within the [0, 1] interval: {}",
            i,
            key.t
        );
    }
}

#[cfg(debug_assertions)]
fn check_equidistance(keys: &[Key<f32, f32>]) {
    for (i, group) in keys.chunks_exact(3).enumerate() {
        let distance_is_same = {
            let right_chunk = group[2].t - group[1].t;
            let left_chunk = group[1].t - group[0].t;

            (right_chunk - left_chunk).abs() < f32::EPSILON
        };

        debug_assert!(
            distance_is_same,
            "Spline points are not equidistant starting from: {}. Keys:\n{:#?}",
            i, keys
        );
    }
}

#[cfg(debug_assertions)]
fn check_1_0_point_exists(keys: &[Key<f32, f32>]) {
    let last_key = keys.last().unwrap();

    debug_assert!(
        (1.0 - last_key.t) < f32::EPSILON,
        "Missing the last point at t = 1.0 of the spline. Keys:\n{:#?}",
        keys
    );
}
