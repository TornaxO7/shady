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
    /// Writes *nothing* into the given buffer.
    fn fetch_snapshot(&mut self, _buf: &mut [f32]) {}
}
