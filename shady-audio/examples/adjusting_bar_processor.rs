use std::num::NonZeroUsize;

use shady_audio::{
    bar_processor::{BarProcessor, Config},
    fetcher::DummyFetcher,
    SampleProcessor,
};

fn main() {
    let sample_processor = SampleProcessor::new(DummyFetcher::new());

    let mut bar_processor = BarProcessor::new(&sample_processor, Config::default());
    bar_processor.process_bars(&sample_processor);

    bar_processor = BarProcessor::new(
        &sample_processor,
        Config {
            amount_bars: NonZeroUsize::new(50).unwrap(),
            ..bar_processor.to_config().clone()
        },
    );

    bar_processor.process_bars(&sample_processor);
}
