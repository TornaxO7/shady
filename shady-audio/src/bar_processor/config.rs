use std::{num::NonZero, ops::Range};

/// Decides which interpolation strategy for the bars.
#[derive(Debug, Clone, Copy, Hash)]
pub enum InterpolationVariant {
    /// No interpolation strategy should be used.
    ///
    /// Only the "supporting bars" (a.k.a. the bars which are picked up for a frequency range) which are calculated are going to be displayed.
    None,

    /// Use the linear interpolation.
    ///
    Linear,

    /// Use the cubic spline interpolation (recommended since it's the smoothest).
    CubicSpline,
}

#[derive(Debug, Clone, Copy)]
pub struct Sensitivity {
    pub min: f32,
    pub max: f32,
}

/// The config options for [crate::BarProcessor].
#[derive(Debug, Clone)]
pub struct Config {
    /// Set the amount of bars which should be created.
    pub amount_bars: NonZero<u16>,

    /// Set the frequency range which the bar processor should consider.
    pub freq_range: Range<NonZero<u16>>,

    /// Decide how the bar values should be interpolated.
    pub interpolation: InterpolationVariant,

    /// Control how fast the bars should adjust to their new height.
    ///
    /// - `min`: How "fast" the bar should jump at least.
    /// - `max`: How "fast" the bar should jump at most.
    ///
    /// `min` and `max` should be within the range `[0, 1]`.
    /// You should play around with those values until you like the sensitivity of the bars.
    pub sensitivity: Sensitivity,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            interpolation: InterpolationVariant::CubicSpline,
            amount_bars: NonZero::new(30).unwrap(),
            freq_range: NonZero::new(50).unwrap()..NonZero::new(10_000).unwrap(),
            sensitivity: Sensitivity { min: 0.1, max: 0.2 },
        }
    }
}
