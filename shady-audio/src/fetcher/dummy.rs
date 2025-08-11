use super::Fetcher;

/// A dummy fetcher which does... nothing.
/// Mainly used for docs and tests.
pub struct DummyFetcher {
    amount_channels: u16,
}

impl DummyFetcher {
    /// Creates a new instance of this struct.
    pub fn new(amount_channels: u16) -> Box<Self> {
        Box::new(Self { amount_channels })
    }
}

impl Fetcher for DummyFetcher {
    fn fetch_samples(&mut self, _buf: &mut [f32]) {}

    fn sample_rate(&self) -> cpal::SampleRate {
        cpal::SampleRate(44_100)
    }

    fn channels(&self) -> u16 {
        self.amount_channels
    }
}
