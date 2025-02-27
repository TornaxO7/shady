//! Config of an [Equalizer].
//!
//! [Equalizer]: crate::equalizer::Equalizer
use std::{
    num::{NonZero, NonZeroU32, NonZeroUsize},
    ops::Range,
};

/// All validation errors of the [Config].
#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    /// Occurs, if you've set [`Config::freq_range`] to an empty range.
    ///
    /// # Example
    /// ```rust
    /// use shady_audio::equalizer::config::Config;
    /// use std::num::NonZeroU32;
    ///
    /// let invalid_range = NonZeroU32::new(10).unwrap()..NonZeroU32::new(10).unwrap();
    /// assert!(invalid_range.is_empty(), "`start` and `end` are equal");
    ///
    /// let config = Config {
    ///     freq_range: invalid_range.clone(),
    ///     ..Default::default()
    /// };
    ///
    /// // the range isn't allowed to be empty!
    /// assert!(config.validate().is_err());
    /// ```
    #[error("Frequency range can't be empty but you gave: {0:?}")]
    EmptyFreqRange(Range<NonZeroU32>),
}

/// Configure an [Equalizer].
///
/// [Equalizer]: crate::equalizer::Equalizer
#[derive(Debug, Clone)]
pub struct Config {
    /// Set the amount bars which should be used.
    pub amount_bars: NonZeroUsize,

    /// Set the frequency range of which the equalizer should listen to for the bars.
    ///
    /// # Example
    /// ```rust
    /// use shady_audio::equalizer::config::Config;
    /// use std::num::NonZeroU32;
    ///
    /// let config = Config {
    ///     // `shady_audio` should only listen to the frequencies starting from 10Hz up to 15_000Hz.
    ///     freq_range: NonZeroU32::new(100).unwrap()..NonZeroU32::new(15_000).unwrap(),
    ///     ..Default::default()
    /// };
    /// ```
    pub freq_range: Range<NonZeroU32>,

    /// The initial sensitivity. In general, just use the default value since it will change anyhow.
    /// But if you are curious:
    ///
    /// - `< 1.0` means that the output of the audio processor are greater than `1.0` and need to be lowered.
    /// - `1.0` means that the output of the audio processor shouldn't change
    /// - `> 1.0` means that the output of the audio processor is smaller than `1.0` and needs to be increased.
    pub init_sensitivity: f32,
}

impl Config {
    /// Checks if the current config is valid or contains any mistakes.
    ///
    /// See [`Error`] to see all possible errors.
    pub fn validate(&self) -> Result<(), Error> {
        if self.freq_range.is_empty() {
            return Err(Error::EmptyFreqRange(self.freq_range.clone()));
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            amount_bars: NonZeroUsize::new(32).unwrap(),
            freq_range: NonZeroU32::new(50).unwrap()..NonZero::new(10_000).unwrap(),
            init_sensitivity: 1.,
        }
    }
}

impl AsRef<Config> for Config {
    fn as_ref(&self) -> &Config {
        self
    }
}
