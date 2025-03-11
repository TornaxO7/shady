use super::Fetcher;

/// A dummy fetcher which does... nothing.
/// Mainly used for docs and tests.
pub struct DummyFetcher;

impl DummyFetcher {
    /// Creates a new instance of this struct.
    pub fn new() -> Box<Self> {
        Box::new(Self)
    }
}

impl Fetcher for DummyFetcher {
    fn fetch_samples(&mut self, _buf: &mut [f32]) {}

    fn sample_rate(&self) -> cpal::SampleRate {
        cpal::SampleRate(44_100)
    }
}
