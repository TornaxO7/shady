use core::f32;
use std::{marker::PhantomData, num::NonZeroUsize, ops::Range};

use cpal::SampleRate;
use tracing::debug;

use crate::{
    config::{self, EqualizerConfig},
    processor::AudioProcessor,
    MAX_HUMAN_FREQUENCY, MIN_HUMAN_FREQUENCY,
};

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("There are not enough bars avai")]
    NotEnoughBars {
        amount_bins: usize,
        amount_bars: usize,
    },

    #[error(transparent)]
    InvalidConfig(#[from] config::Error),
}

#[derive(Debug, Clone)]
struct State {
    sample_rate: SampleRate,
    fft_size: usize,
    sensitivity: f32,

    amount_bars: usize,
    freq_range: Range<u32>,
}

#[derive(Debug)]
pub struct Equalizer<P> {
    bar_values: Box<[f32]>,
    bar_ranges: Box<[Range<usize>]>,
    started_falling: Box<[bool]>,

    state: State,

    _phantom_data: PhantomData<P>,
}

impl<P> Equalizer<P> {
    pub fn new(config: &EqualizerConfig, processor: &AudioProcessor<P>) -> Result<Self, Error> {
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

    pub fn set_bars(&mut self, amount_bars: NonZeroUsize) {
        self.state.amount_bars = usize::from(amount_bars);
        *self = Self::inner_new(self.state.clone()).unwrap();
    }

    pub fn process(&mut self, audio: &AudioProcessor<P>) -> &[f32] {
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

    fn inner_new(state: State) -> Result<Self, Error> {
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

            if amount_bins >= state.amount_bars {
                return Err(Error::NotEnoughBars {
                    amount_bins,
                    amount_bars: state.amount_bars,
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
