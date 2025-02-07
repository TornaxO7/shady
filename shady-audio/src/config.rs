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

#[derive(thiserror::Error, Debug, Clone)]
pub enum ConfigError {
    #[error("Frequency rang can't be empty but you gave: {0:?}")]
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
///
/// // ... do some wild stuff ...
///
/// // maybe... we would like to change something :>
/// // Let it fetch the latest data faster.
/// let new_config = ShadyAudioConfig {
///     refresh_time: Duration::from_millis(50),
///     .. Default::default()
/// };
///
/// shady_audio.update_config(new_config);
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

    pub amount_bars: NonZeroUsize,

    pub freq_range: Range<NonZeroU32>,
}

impl ShadyAudioConfig {
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
            freq_range: NonZeroU32::new(50).unwrap()..NonZero::new(15_000).unwrap(),
        }
    }
}
