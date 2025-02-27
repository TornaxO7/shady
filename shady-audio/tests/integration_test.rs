use std::num::NonZeroUsize;

use shady_audio::{
    equalizer::{config::Config, Equalizer},
    fetcher::DummyFetcher,
    processor::AudioProcessor,
};

struct Tag;

#[test]
fn correct_amount_bars() {
    let amount_bars = 5;

    let audio: AudioProcessor<Tag> = AudioProcessor::new(DummyFetcher::new());
    let mut equalizer = Equalizer::new(
        Config {
            amount_bars: NonZeroUsize::new(amount_bars).unwrap(),
            ..Default::default()
        },
        &audio,
    )
    .unwrap();

    let bars = equalizer.get_bars(&audio);

    assert_eq!(bars.len(), amount_bars);
}
