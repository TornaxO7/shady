`shady-audio` is the audio backend for the other [shady tools].
Its interface let's you easily fetch the frequency presence of the given audio source
by selecting an audio source which implemented the `Fetcher` trait.

# Example

```rust
use std::num::NonZeroUsize;

use shady_audio::{ShadyAudio, fetcher::DummyFetcher, config::ShadyAudioConfig};

let mut audio = {
    // A fetcher feeds new samples to `ShadyAudio` which processes it
    let fetcher = DummyFetcher::new();

    // configure the behaviour of `ShadyAudio`
    let config = ShadyAudioConfig {
        amount_bars: NonZeroUsize::new(10).unwrap(),
        ..Default::default()
    };

    ShadyAudio::new(fetcher, config).unwrap()
};

// just retrieve the bars.
// ShadyAudio takes care of the rest:
//   - fetching new samples from the fetcher
//   - normalize the values within the range [0, 1]
//   - etc.
assert_eq!(audio.get_bars().len(), 10);

// change the amount of bars you'd like to have
audio.set_bars(NonZeroUsize::new(20).unwrap());
assert_eq!(audio.get_bars().len(), 20);
```

[shady tools]: https://github.com/TornaxO7/shady
