use std::marker::PhantomData;

use cpal::SampleRate;
use realfft::{num_complex::Complex32, RealFftPlanner};

use crate::fetcher::Fetcher;

/// Processes the samples of the given fetcher.
///
/// The structs fetches the samples of the fetcher and creates the frequency bins which should then be further
/// processed by the equalizers.
///
/// # `Tag`
/// The `Tag` generic is there to avoid mixing up multiple equalizers with multiple `AudioProcessor`s.
/// In code, it avoids the following case:
///
/// ```rust
/// use shady_audio::{
///     equalizer::{config::EqualizerConfig, Equalizer},
///     fetcher::DummyFetcher,
///     AudioProcessor,
/// };
///
/// struct Tag1;
/// struct Tag2;
///
/// let audio1: AudioProcessor<Tag1> = AudioProcessor::new(DummyFetcher::new());
/// let audio2: AudioProcessor<Tag2> = AudioProcessor::new(DummyFetcher::new());
///
/// let mut equalizer = Equalizer::new(EqualizerConfig::default(), &audio1).unwrap();
///
/// // the equalizer is only allowed to create the bars of `audio1` or in other words: Only the `AudioProcessor`
/// // with the tag `Tag1`
/// equalizer.get_bars(&audio1);
///
/// // Uncommenting this wouldn't compile
/// // equalizer.get_bars(&audio2);
/// ```
pub struct AudioProcessor<Tag> {
    planner: RealFftPlanner<f32>,
    hann_window: Box<[f32]>,

    scratch_buffer: Box<[Complex32]>,
    fft_out: Box<[Complex32]>,
    fft_in: Box<[f32]>,
    fft_in_raw: Box<[f32]>,

    fft_size: usize,

    fetcher: Box<dyn Fetcher>,
    sample_buffer: Vec<f32>,

    _phantom_data: PhantomData<Tag>,
}

impl<Tag> AudioProcessor<Tag> {
    /// Create a new instance of this struct.
    pub fn new(fetcher: Box<dyn Fetcher>) -> Self {
        let fft_size = {
            let sample_rate = fetcher.sample_rate().0;
            let factor = if sample_rate < 8_125 {
                1
            } else if sample_rate <= 16_250 {
                2
            } else if sample_rate <= 32_500 {
                4
            } else if sample_rate <= 75_000 {
                8
            } else if sample_rate <= 150_000 {
                16
            } else if sample_rate <= 300_000 {
                32
            } else {
                64
            };

            factor * 128
        };
        let fft_out_size = fft_size / 2 + 1;

        let hann_window = apodize::hanning_iter(fft_size)
            .map(|val| val as f32)
            .collect::<Vec<f32>>()
            .into_boxed_slice();

        let sample_buffer = Vec::with_capacity(fft_size);

        let fft_in = vec![0.; fft_size].into_boxed_slice();
        let fft_in_raw = vec![0.; fft_size].into_boxed_slice();
        let scratch_buffer = vec![Complex32::ZERO; fft_out_size].into_boxed_slice();
        let fft_out = vec![Complex32::ZERO; fft_out_size].into_boxed_slice();

        Self {
            planner: RealFftPlanner::new(),
            hann_window,
            scratch_buffer,
            fft_out,
            fft_in,
            fft_in_raw,

            fft_size,

            fetcher,
            sample_buffer,

            _phantom_data: PhantomData,
        }
    }

    /// Fetches the next batch of the internal fetcher and processes them.
    /// The processed samples can be retrieved by using the [AudioProcessor::fft_out] method.
    ///
    /// # Example
    /// See [AudioProcessor::fft_out].
    #[inline]
    pub fn process(&mut self) {
        self.fetch_new_samples();
        let new_len = self.sample_buffer.len().min(self.fft_size);

        self.fft_in_raw
            .copy_within(..self.fft_size - new_len, new_len);
        self.fft_in_raw[..new_len].copy_from_slice(&&self.sample_buffer[..new_len]);

        for (i, &sample) in self.fft_in_raw.iter().enumerate() {
            self.fft_in[i] = sample * self.hann_window[i];
        }

        let fft = self.planner.plan_fft_forward(self.fft_size);
        fft.process_with_scratch(
            &mut self.fft_in,
            self.fft_out.as_mut(),
            self.scratch_buffer.as_mut(),
        )
        .unwrap();
    }

    /// Returns the processed samples of the last time you called [AudioProcessor:process].
    ///
    /// # Example
    /// ```
    /// use shady_audio::{AudioProcessor, fetcher::DummyFetcher};
    ///
    /// struct Tag;
    ///
    /// let mut audio: AudioProcessor<Tag> = AudioProcessor::new(DummyFetcher::new());
    ///
    /// // fetch new samples and process them
    /// audio.process();
    ///
    /// // get the processed output
    /// let out = audio.fft_out().clone();
    ///
    /// // as long as you don't call `audio.process()` again, the last result will be returned
    /// assert_eq!(out, audio.fft_out());
    /// ```
    pub fn fft_out(&self) -> &[Complex32] {
        &self.fft_out
    }

    fn fetch_new_samples(&mut self) {
        self.fetcher.fetch_samples(&mut self.sample_buffer);
        self.sample_buffer.clear();
    }
}

// Crate internal public methods
impl<T> AudioProcessor<T> {
    pub(crate) fn fft_size(&self) -> usize {
        self.fft_size
    }

    pub(crate) fn sample_rate(&self) -> SampleRate {
        self.fetcher.sample_rate()
    }
}
