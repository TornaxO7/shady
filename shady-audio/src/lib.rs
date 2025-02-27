//! # Description
//! A crate which simplifies the data management of audio sources to be easily able
//! to retrieve the frequency powers of the source.
//!
//! ### [cpal]
//!
//! This crate also re-exports [cpal] so there's no need to add [cpal] exclusively
//! to your dependency list.
//!
//! # How to get started
//! The main usage can be seen in the example below.
//! Take a look to the available methods of [ShadyAudio] if you would like to change some properties of it (like frequency range or amount of bars).
//!
//! # Example
//! ```rust
//! use std::num::NonZeroUsize;
//!
//! use shady_audio::{ShadyAudio, fetcher::DummyFetcher, config::ShadyAudioConfig};
//!
//! let mut audio = {
//!     // A fetcher feeds new samples to `ShadyAudio` which processes it
//!     let fetcher = DummyFetcher::new();
//!
//!     // configure the behaviour of `ShadyAudio`
//!     let config = ShadyAudioConfig {
//!         amount_bars: NonZeroUsize::new(10).unwrap(),
//!         ..Default::default()
//!     };
//!
//!     ShadyAudio::new(fetcher, config).unwrap()
//! };
//!
//! // just retrieve the bars.
//! // ShadyAudio takes care of the rest:
//! //   - fetching new samples from the fetcher
//! //   - normalize the values within the range [0, 1]
//! //   - etc.
//! assert_eq!(audio.get_bars().len(), 10);
//!
//! // change the amount of bars you'd like to have
//! audio.set_bars(NonZeroUsize::new(20).unwrap());
//! assert_eq!(audio.get_bars().len(), 20);
//! ```
pub mod config;
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
