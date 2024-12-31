//! Module to configure the behaviour of [ShadyAudio].
//!
//! [ShadyAudio]: crate::ShadyAudio
use std::time::Duration;

/// Default value for [ShadyAudioConfig.refresh_time].
/// Set to `100` millis.
///
/// [ShadyAudioConfig.refresh_time]: struct.ShadyAudioConfig.html#structfield.refresh_time
pub const DEFAULT_REFRESH_TIME: Duration = Duration::from_millis(100);

/// Configure the behaviour of [ShadyAudio] by setting the appropriate values in this struct
/// and give it to [ShadyAudio].
///
/// # Example
/// ```rust
/// use shady_audio::{fetcher::DummyFetcher, ShadyAudio, config::ShadyAudioConfig};
/// use std::time::Duration;
///
/// fn main() {
///     let mut shady_audio = ShadyAudio::new(DummyFetcher::boxed(), ShadyAudioConfig::default());
///
///     // ... do some wild stuff ...
///
///     // maybe... we would like to change something :>
///     // Let it fetch the latest data faster.
///     let new_config = ShadyAudioConfig {
///         refresh_time: Duration::from_millis(50),
///         .. Default::defaults()
///     };
///
///     shady_audio.update_config(new_config);
/// }
/// ```
///
/// [ShadyAudio]: crate::ShadyAudio
#[derive(Debug, Clone, Copy, Hash)]
pub struct ShadyAudioConfig {
    /// The duration how long shady should wait, until it should fetch
    /// from the audio source again.
    ///
    /// The rule is basically: The higher the duration, the slower your
    /// music visualizer becomes.
    ///
    /// # Default
    /// See [DEFAULT_REFRESH_TIME].
    pub refresh_time: Duration,
}

impl Default for ShadyAudioConfig {
    fn default() -> Self {
        Self {
            refresh_time: DEFAULT_REFRESH_TIME,
        }
    }
}
