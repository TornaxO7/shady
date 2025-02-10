//! Module to configure the behaviour of [ShadyAudio].
//!
//! [ShadyAudio]: crate::ShadyAudio
use std::{
    num::{NonZero, NonZeroU32, NonZeroUsize},
    ops::Range,
    time::Duration,
};

/// Default value for [ShadyAudioConfig.refresh_time].
/// Set to `100` millis.
///
/// [ShadyAudioConfig.refresh_time]: struct.ShadyAudioConfig.html#structfield.refresh_time
pub const DEFAULT_REFRESH_TIME: Duration = Duration::from_millis(100);

/// Contains all invalid configurations of [`ShadyAudioConfig`].
#[derive(thiserror::Error, Debug, Clone)]
pub enum ConfigError {
    /// Occurs, if you've set [`ShadyAudioConfig::freq_range`] to an empty range.
    ///
    /// # Example
    /// ```rust
    /// use shady_audio::config::{ShadyAudioConfig, ConfigError};
    /// use std::num::NonZeroU32;
    ///
    /// let invalid_range = NonZeroU32::new(10).unwrap()..NonZeroU32::new(10).unwrap();
    /// assert!(invalid_range.is_empty(), "`start` and `end` are equal");
    ///
    /// let config = ShadyAudioConfig {
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

/// Configure the behaviour of [ShadyAudio] by setting the appropriate values in this struct
/// and give it to [ShadyAudio].
///
/// # Example
/// ```rust
/// use shady_audio::{fetcher::DummyFetcher, ShadyAudio, config::ShadyAudioConfig};
/// use std::time::Duration;
///
/// let mut shady_audio = ShadyAudio::new(DummyFetcher::new(), ShadyAudioConfig::default());
/// ```
///
/// [ShadyAudio]: crate::ShadyAudio
#[derive(Debug, Clone, Hash)]
pub struct ShadyAudioConfig {
    /// The duration how long shady should wait, until it should fetch
    /// from the audio source again.
    ///
    /// The rule is basically: The higher the duration, the slower your
    /// music visualizer becomes.
    ///
    /// # Default
    /// See [DEFAULT_REFRESH_TIME
    pub refresh_time: Duration,

    /// Set the amount bars which should be used.
    pub amount_bars: NonZeroUsize,

    /// Set the frequency range of which `shady-audio` should listen to for the bars.
    ///
    /// # Example
    /// ```rust
    /// use shady_audio::config::ShadyAudioConfig;
    /// use std::num::NonZeroU32;
    ///
    /// let config = ShadyAudioConfig {
    ///     // `shady_audio` should only listen to the frequencies starting from 10Hz up to 15_000Hz.
    ///     freq_range: NonZeroU32::new(100).unwrap()..NonZeroU32::new(15_000).unwrap(),
    ///     ..Default::default()
    /// };
    /// ```
    pub freq_range: Range<NonZeroU32>,
}

impl ShadyAudioConfig {
    /// Checks if the current config is valid or contains any mistakes.
    ///
    /// See [`ConfigError`] to see all possible errors.
    pub fn validate(&self) -> Result<(), Vec<ConfigError>> {
        let mut errors = Vec::new();

        if self.freq_range.is_empty() {
            errors.push(ConfigError::EmptyFreqRange(self.freq_range.clone()))
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(())
    }
}

impl Default for ShadyAudioConfig {
    fn default() -> Self {
        Self {
            refresh_time: DEFAULT_REFRESH_TIME,
            amount_bars: NonZeroUsize::new(32).unwrap(),
            freq_range: NonZeroU32::new(50).unwrap()..NonZero::new(10_000).unwrap(),
        }
    }
}
