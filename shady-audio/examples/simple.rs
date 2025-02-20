use std::{num::NonZeroUsize, time::Duration};

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

    loop {
        audio.get_bars();
        std::thread::sleep(Duration::from_millis(1000 / 60));
    }
}
