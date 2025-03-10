use shady_audio::{
    bar_processor::{BarProcessor, Config},
    fetcher::DummyFetcher,
    interpolation::CubicSpline,
    SampleProcessor,
};

fn main() {
    let mut sample_processor = SampleProcessor::new(DummyFetcher::new());
    sample_processor.process_next_samples();

    let mut bar_processor: BarProcessor<CubicSpline> =
        BarProcessor::new(&sample_processor, Config::default());

    bar_processor.process_bars(&sample_processor);
}
