//! Contains the equalizer which creates the bars from the output of the processor.
pub mod config;

use core::f32;
use std::{marker::PhantomData, num::NonZeroUsize, ops::Range};

use cpal::SampleRate;
use tracing::debug;

use crate::{processor::AudioProcessor, MAX_HUMAN_FREQUENCY, MIN_HUMAN_FREQUENCY};

/// The errors which can occur while configuring the [Equalizer].
#[derive(thiserror::Error, Debug, Clone)]
pub enum EqualizerError {
    /// The sample rate of the fetcher of the audio processor is too low.
    ///
    /// # The bigger context
    /// The idea is here that the sample rate basically decides how many frequencies we
    /// can distinguish so it musn't be lower than the amount of your requested bars for the equalizer.
    #[error(
        "The sample rate of the fetcher is too low. It must be at least {min_sample_rate} Hz."
    )]
    TooLowSampleRate { min_sample_rate: usize },

    /// The given config is invalid.
    #[error(transparent)]
    InvalidConfig(#[from] config::ConfigError),
}

#[derive(Debug, Clone)]
struct State {
    sample_rate: SampleRate,
    fft_size: usize,
    sensitivity: f32,

    amount_bars: usize,
    freq_range: Range<u32>,
}

/// The main struct to create the values for the bars from a processor.
///
/// The `Tag` generic forces you to use only one specifique audio processor instead of from multiple due to invariants.
#[derive(Debug)]
pub struct Equalizer<Tag> {
    bar_values: Box<[f32]>,
    bar_ranges: Box<[Range<usize>]>,
    started_falling: Box<[bool]>,

    state: State,

    _phantom_data: PhantomData<Tag>,
}

impl<Tag> Equalizer<Tag> {
    /// Create a new equalizer for the given audio processor.
    pub fn new(
        config: impl AsRef<config::EqualizerConfig>,
        processor: &AudioProcessor<Tag>,
    ) -> Result<Self, EqualizerError> {
        let config = config.as_ref();
        assert!(
            processor.sample_rate().0 > 0,
            "fetcher has invalid sample rate of 0"
        );

        config.validate()?;

        let state = State {
            sample_rate: processor.sample_rate(),
            fft_size: processor.fft_size(),
            sensitivity: config.init_sensitivity,
            amount_bars: usize::from(config.amount_bars),
            freq_range: u32::from(config.freq_range.start)..u32::from(config.freq_range.end),
        };

        Self::inner_new(state)
    }

    /// Update the amount of bars it should produce.
    ///
    /// # Example
    /// ```rust
    /// use shady_audio::{
    ///     equalizer::{Equalizer, config::EqualizerConfig},
    ///     fetcher::DummyFetcher,
    ///     processor::AudioProcessor,
    /// };
    /// use std::num::NonZeroUsize;
    ///
    /// struct Tag;
    ///
    /// let amount_bars = 5;
    ///
    /// let mut audio: AudioProcessor<Tag> = AudioProcessor::new(DummyFetcher::new());
    /// let mut equalizer = Equalizer::new(EqualizerConfig {
    ///         amount_bars: NonZeroUsize::new(amount_bars).unwrap(),
    ///         ..Default::default()
    ///     }, &audio
    /// )
    /// .unwrap();
    ///
    /// assert_eq!(equalizer.get_bars(&audio).len(), amount_bars);
    /// ```
    pub fn set_bars(&mut self, amount_bars: NonZeroUsize) {
        self.state.amount_bars = usize::from(amount_bars);
        *self = Self::inner_new(self.state.clone()).unwrap();
    }

    /// Return the bars with their values.
    ///
    /// Each bar value tries to stay within the range `[0, 1]` but it could happen that there are some spikes which go above `1`. However it will slowly normalize itself back to `1`.
    pub fn get_bars(&mut self, audio: &AudioProcessor<Tag>) -> &[f32] {
        let fft_out = audio.fft_out();
        let mut overshoot = false;
        let mut is_silent = true;

        for (i, range) in self.bar_ranges.iter().cloned().enumerate() {
            let prev_magnitude = self.bar_values[i];
            let next_magnitude: f32 = {
                let raw_bar_val = fft_out[range]
                    .iter()
                    .map(|out| {
                        let mag = out.norm();
                        if mag > 0. {
                            is_silent = false;
                        }
                        mag
                    })
                    .max_by(|a, b| a.total_cmp(b))
                    .unwrap();

                self.state.sensitivity
                    * raw_bar_val
                    * 10f32.powf((i as f32 / self.bar_values.len() as f32) - 1.1)
            };

            debug_assert!(!prev_magnitude.is_nan());
            debug_assert!(!next_magnitude.is_nan());

            let rel_change = next_magnitude / prev_magnitude;
            if is_silent {
                self.bar_values[i] *= 0.75;
                self.started_falling[i] = false;
            } else {
                let was_already_falling = self.started_falling[i];
                if next_magnitude < prev_magnitude && !was_already_falling {
                    self.started_falling[i] = true;
                    self.bar_values[i] += (next_magnitude - prev_magnitude) * 0.1;
                } else {
                    self.started_falling[i] = false;
                    self.bar_values[i] +=
                        (next_magnitude - prev_magnitude) * rel_change.clamp(0.05, 0.2);
                }
            }

            if self.bar_values[i] > 1. {
                overshoot = true;
            }
        }

        if overshoot {
            self.state.sensitivity *= 0.98;
        } else if !is_silent {
            self.state.sensitivity *= 1.002;
        }

        &self.bar_values
    }

    fn inner_new(state: State) -> Result<Self, EqualizerError> {
        let bar_values = vec![0.; state.amount_bars].into_boxed_slice();
        let started_falling = vec![false; state.amount_bars].into_boxed_slice();

        let bar_ranges = {
            let freq_resolution = state.sample_rate.0 as f32 / state.fft_size as f32;
            debug!("Freq resolution: {}", freq_resolution);

            let weights = (0..state.amount_bars)
                .map(|index| exp_fun(index as f32 / state.amount_bars as f32))
                .collect::<Vec<f32>>();
            debug!("Weights: {:?}", weights);

            // the relevant index range of the fft output which we should use for the bars
            let bin_range = Range {
                start: ((state.freq_range.start as f32 / freq_resolution) as usize).max(1),
                end: (state.freq_range.end as f32 / freq_resolution).ceil() as usize,
            };
            let amount_bins = bin_range.len();
            debug!("Bin range: {:?}", bin_range);
            debug!("Available bins: {}", amount_bins);

            if amount_bins < state.amount_bars {
                return Err(EqualizerError::TooLowSampleRate {
                    min_sample_rate: state.fft_size,
                });
            }

            let ranges = {
                let mut cut_offs = Vec::with_capacity(state.amount_bars);
                let mut start = 0;

                for weight in weights {
                    let mut end = ((weight / MAX_HUMAN_FREQUENCY as f32) * amount_bins as f32)
                        .ceil() as usize;
                    if start >= end {
                        end = start + 1;
                    }

                    cut_offs.push(start..end);
                    start = end;
                }
                // let the last bar use every resulting bar
                let last_range = cut_offs
                    .last_mut()
                    .expect("There's at least one range/bar.");
                last_range.end = amount_bins;

                cut_offs
            };
            tracing::debug!("Bin ranges: {:?}", ranges);

            ranges.into_boxed_slice()
        };

        Ok(Self {
            bar_values,
            bar_ranges,
            started_falling,
            state,
            _phantom_data: PhantomData,
        })
    }
}

fn exp_fun(x: f32) -> f32 {
    debug_assert!(0. <= x);
    debug_assert!(x <= 1.);

    let max_mel_value = mel(MAX_HUMAN_FREQUENCY as f32);
    let min_mel_value = mel(MIN_HUMAN_FREQUENCY as f32);

    // map [0, 1] => [min-mel-value, max-mel-value]
    let mapped_x = x * (max_mel_value - min_mel_value) + min_mel_value;
    inv_mel(mapped_x)
}

// https://en.wikipedia.org/wiki/Mel_scale
fn mel(x: f32) -> f32 {
    debug_assert!(MIN_HUMAN_FREQUENCY as f32 <= x);
    debug_assert!(x <= MAX_HUMAN_FREQUENCY as f32);

    2595. * (1. + x / 700.).log10()
}

fn inv_mel(x: f32) -> f32 {
    let min_mel_value = mel(MIN_HUMAN_FREQUENCY as f32);
    let max_mel_value = mel(MAX_HUMAN_FREQUENCY as f32);

    debug_assert!(min_mel_value <= x);
    debug_assert!(x <= max_mel_value);

    700. * (10f32.powf(x / 2595.) - 1.)
}
