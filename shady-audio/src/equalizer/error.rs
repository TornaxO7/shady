/// The errors which can occur while configuring the [Equalizer].
#[derive(thiserror::Error, Debug, Clone)]
pub enum EqualizerError {
    /// The sample rate of the fetcher of the audio processor is too low.
    ///
    /// # The bigger context
    /// The idea is here that the sample rate basically decides how many frequencies we
    /// can distinguish so it musn't be lower than the amount of your requested bars for the equalizer.
    #[error(
        "The sample rate of the fetcher is too low. It must be at least {min_sample_rate} Hz."
    )]
    TooLowSampleRate { min_sample_rate: usize },

    /// The given config is invalid.
    #[error(transparent)]
    InvalidConfig(#[from] config::ConfigError),
}
