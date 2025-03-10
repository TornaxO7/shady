use std::{
    num::{NonZero, NonZeroU16, NonZeroUsize},
    ops::Range,
};

use crate::interpolation::InterpolationVariant;

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
