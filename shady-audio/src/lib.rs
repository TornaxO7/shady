//! # Description
//! A crate which simplifies the data management of audio sources to be easily able
//! to retrieve the frequency powers of the source.
//!
//! ### [cpal]
//!
//! This crate also re-exports [cpal] so there's no need to add [cpal] exclusively
//! to your dependency list.
//!
//! # Example
//!
//! ## Simple workflow
//! A simple workflow can look like this:
//! ```
//! use shady_audio::{SampleProcessor, BarProcessor, BarProcessorConfig, fetcher::DummyFetcher};
//!
//! let mut sample_processor = SampleProcessor::new(DummyFetcher::new());
//! // Note: The bar procesor is intended to only work with the given sample processor.
//! let mut bar_processor = BarProcessor::new(
//!     &sample_processor,
//!     BarProcessorConfig::default()
//! );
//!
//! loop {
//!     // let the sample processor process the next batch of samples
//!     sample_processor.process_next_samples();
//!
//!     // let the bar processor convert the samples into "bar-values"
//!     // which are tried to be set in the range of `[0, 1]`.
//!     let bars = bar_processor.process_bars(&sample_processor);
//!
//!     break;
//! }
//! ```
//!
//! ## Multiple bar processors
//! You can also create multiple [BarProcessor]s with different configs.
//!
//! ```
//! use std::num::NonZero;
//! use shady_audio::{SampleProcessor, BarProcessor, BarProcessorConfig, fetcher::DummyFetcher};
//!
//! let mut sample_processor = SampleProcessor::new(DummyFetcher::new());
//!
//! let mut bar_processor = BarProcessor::new(
//!     &sample_processor,
//!     BarProcessorConfig {
//!         amount_bars: NonZero::new(20).unwrap(),
//!         ..Default::default()
//!     }
//! );
//! let mut bar_processor2 = BarProcessor::new(
//!     &sample_processor,
//!     BarProcessorConfig {
//!         amount_bars: NonZero::new(10).unwrap(),
//!         ..Default::default()
//!     }
//! );
//!
//! loop {
//!     // the sample processor needs to compute the new samples only once
//!     // for both bar processors (to reduce computation)
//!     sample_processor.process_next_samples();
//!
//!     let bars = bar_processor.process_bars(&sample_processor);
//!     let bars2 = bar_processor2.process_bars(&sample_processor);
//!
//!     assert_eq!(bars.len(), 20);
//!     assert_eq!(bars2.len(), 10);
//!
//!     break;
//! }
//! ```
//!
//! ## Change config of bar processors
//! You have to create a new [BarProcessor] if you want to change it's behaviour like the amount
//! of bars it should output.
//! ```
//! use std::num::NonZero;
//! use shady_audio::{SampleProcessor, BarProcessor, BarProcessorConfig, fetcher::DummyFetcher};
//!
//! let mut sample_processor = SampleProcessor::new(DummyFetcher::new());
//! let mut bar_processor = BarProcessor::new(
//!     &sample_processor,
//!     BarProcessorConfig {
//!         amount_bars: NonZero::new(20).unwrap(),
//!     ..Default::default()
//!     }
//! );
//!
//! assert_eq!(bar_processor.process_bars(&sample_processor).len(), 20);
//!
//! // change the amount of bars
//! bar_processor = BarProcessor::new(
//!     &sample_processor,
//!     BarProcessorConfig {
//!         amount_bars: NonZero::new(10).unwrap(),
//!        ..bar_processor.config().clone()
//!     }
//! );
//!
//! assert_eq!(bar_processor.process_bars(&sample_processor).len(), 10);
//! ```
pub mod fetcher;

mod bar_processor;
mod interpolation;
mod sample_processor;

pub use bar_processor::{BarProcessor, BarProcessorConfig, InterpolationVariant};
pub use cpal;
pub use easing_function::easings::StandardEasing;
pub use sample_processor::SampleProcessor;

use cpal::SampleRate;

type Hz = u16;

/// The minimal frequency which humans can here (roughly)
/// See: <https://en.wikipedia.org/wiki/Hearing_range>
pub const MIN_HUMAN_FREQUENCY: Hz = 20;

/// The maximal frequency which humans can here (roughly)
/// See: <https://en.wikipedia.org/wiki/Hearing_range>
pub const MAX_HUMAN_FREQUENCY: Hz = 20_000;

/// The default sample rate for a fetcher.
/// Fetchers are allowed to use this for orientation.
pub const DEFAULT_SAMPLE_RATE: SampleRate = SampleRate(44_100);
