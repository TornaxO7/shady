use std::{
    num::{NonZero, NonZeroU16, NonZeroUsize},
    ops::Range,
};

/// Decides which interpolation strategy should be used.
#[derive(Debug, Clone, Copy, Hash)]
pub enum InterpolationVariant {
    /// No interpolation strategy should be used.
    ///
    /// Only the supporting bars which are calculated are going to be displayed.
    None,

    /// Use the linear interpolation.
    ///
    Linear,

    /// Use the cubic spline interpolation.
    CubicSpline,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub amount_bars: NonZeroUsize,
    pub freq_range: Range<NonZeroU16>,
    pub interpolation: InterpolationVariant,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            interpolation: InterpolationVariant::CubicSpline,
            amount_bars: NonZero::new(30).unwrap(),
            freq_range: NonZero::new(50).unwrap()..NonZero::new(10_000).unwrap(),
        }
    }
}
