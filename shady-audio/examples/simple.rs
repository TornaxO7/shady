use shady_audio::{fetcher::DummyFetcher, BarProcessor, BarProcessorConfig, SampleProcessor};

fn main() {
    let mut sample_processor = SampleProcessor::new(DummyFetcher::new());
    sample_processor.process_next_samples();

    let mut bar_processor = BarProcessor::new(
        &sample_processor,
        Box::new(easing_function::easings::Linear),
        BarProcessorConfig::default(),
    );
    bar_processor.process_bars(&sample_processor);
}
