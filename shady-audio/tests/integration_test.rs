use std::num::NonZeroUsize;

use shady_audio::{fetcher::DummyFetcher, ShadyAudio};

#[test]
fn general() {
    let amount_bars = 5;

    let mut audio = {
        let fetcher = DummyFetcher::new();

        ShadyAudio::new(
            fetcher,
            shady_audio::ShadyAudioConfig {
                amount_bars: NonZeroUsize::new(amount_bars).unwrap(),
                ..Default::default()
            },
        )
        .unwrap()
    };

    let bars = audio.get_bars();

    assert_eq!(bars.len(), amount_bars);
}
