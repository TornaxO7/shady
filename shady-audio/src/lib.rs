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
//! ```rust
//! use shady_audio::{
//!     equalizer::{Equalizer, config::EqualizerConfig},
//!     fetcher::DummyFetcher,
//!     processor::AudioProcessor,
//! };
//!
//! struct Tag;
//!
//! // create the audio processors
//! let mut audio: AudioProcessor<Tag> = AudioProcessor::new(DummyFetcher::new());
//!
//! // now create for each processor an equalizer
//! let mut equalizer = Equalizer::new(EqualizerConfig::default(), &audio).unwrap();
//!
//! // let the processor process the next batch
//! audio.process();
//!
//! // now you can retrieve the bars from the equalizer
//! equalizer.get_bars(&audio);
//!
//! // NOTE: If you uncomment the lines after `==` it won't compile.
//! // `equalizer` is only allowed to process the data from the processor with the tag `Tag`.
//! // However, you can create any amounut of equalizer with different settings (for example different amount of bars) which are reading from the
//! // same audio processor with the same tag.
//! // ===
//! // struct Tag2;
//! // let _audio2: AudioProcessor<Tag2> = AudioProcessor::new(DummyFetcher::new());
//! // equalizer.get_bars(&_audio2);
//! ```
pub mod equalizer;
pub mod fetcher;
pub mod processor;

pub use cpal;

use cpal::SampleRate;

type Hz = u32;

/// The minimal frequency which humans can here (roughly)
/// See: <https://en.wikipedia.org/wiki/Hearing_range>
pub const MIN_HUMAN_FREQUENCY: Hz = 20;

/// The maximal frequency which humans can here (roughly)
/// See: <https://en.wikipedia.org/wiki/Hearing_range>
pub const MAX_HUMAN_FREQUENCY: Hz = 20_000;

/// The default sample rate for a fetcher.
/// Fetchers are allowed to use this for orientation.
pub const DEFAULT_SAMPLE_RATE: SampleRate = SampleRate(44_100);
