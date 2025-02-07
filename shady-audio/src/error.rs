#[derive(thiserror::Error, Debug, Clone, Copy)]
pub enum Error {
    #[error("Frequency range can't be empty")]
    EmptyFrequencyRange,
}
