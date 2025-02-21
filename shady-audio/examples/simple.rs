use std::num::NonZeroUsize;

use shady_audio::{fetcher::DummyFetcher, ShadyAudio};

fn main() {
    let mut audio = ShadyAudio::new(
        DummyFetcher::new(),
        shady_audio::config::ShadyAudioConfig {
            amount_bars: NonZeroUsize::new(5).unwrap(),
            ..Default::default()
        },
    )
    .unwrap();

    for _ in 0..1000 {
        audio.get_bars();
    }
}
