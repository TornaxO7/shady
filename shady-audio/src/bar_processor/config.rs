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
    /// It has to be within the range `[0, 1]`.
    ///
    /// The smaller the value, the slower a height change per bar happens.
    /// The higher the value, the "more" the bar jumps up and down.
    /// So in general the rule of thumb is: The more often you call the `BarProcessor` the smaller
    /// this option needs to be.
    pub sensitivity: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            interpolation: InterpolationVariant::CubicSpline,
            amount_bars: NonZero::new(30).unwrap(),
            freq_range: NonZero::new(50).unwrap()..NonZero::new(10_000).unwrap(),
            sensitivity: 0.2,
        }
    }
}
