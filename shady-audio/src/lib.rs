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
pub mod bar_processor;
pub mod fetcher;

mod interpolation;
mod sample_processor;

pub use cpal;
pub use sample_processor::SampleProcessor;

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
