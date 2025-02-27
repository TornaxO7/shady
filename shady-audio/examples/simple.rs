use shady_audio::{
    equalizer::{config::EqualizerConfig, Equalizer},
    fetcher::DummyFetcher,
    AudioProcessor,
};

struct Tag;

fn main() {
    // create the audio processors
    let mut audio: AudioProcessor<Tag> = AudioProcessor::new(DummyFetcher::new());

    // now create for each processor an equalizer
    let mut equalizer = Equalizer::new(EqualizerConfig::default(), &audio).unwrap();

    // let the processor process the next batch
    audio.process();

    // now you can retrieve the bars from the equalizer
    equalizer.get_bars(&audio);

    // NOTE: If you uncomment the lines after `==` it won't compile.
    // `equalizer` is only allowed to process the data from the processor with the tag `Tag`.
    // ===
    // struct Tag2;
    // let _audio2: AudioProcessor<Tag2> = AudioProcessor::new(DummyFetcher::new());
    // equalizer.get_bars(&_audio2);
}
