`shady-audio` is the audio backend for the other [shady tools].
Its interface let's you easily fetch the frequency presence of the given audio source
by selecting an audio source which implemented the `Fetcher` trait.

# Example

```rust
use std::num::NonZeroUsize;
use shady_audio::{ShadyAudio, fetcher::DummyFetcher, config::ShadyAudioConfig};

fn main() {
    let mut audio = {
        let fetcher = DummyFetcher::new();
        let config = ShadyAudioConfig::default();
        ShadyAudio::new(fetcher, config)
    };
    // Retrieve a spline which you can use, to get any points from the frequancy bands of your audio fetcher.
    // `shady-audio` will take care of the rest. Let it be
    //   - gravity effect
    //   - smooth transition
    //   - etc.
    let spline = audio.get_spline();

    // All relevant points of the spline are stored within the range [0, 1].
    // Since we're currently using the [DummyFetcher] our spline equals the function `f(x) = 0`:
    assert_eq!(spline.sample(0.0), Some(0.0));
    assert_eq!(spline.sample(0.5), Some(0.0));
    // actually for some reason, `splines::Spline` returns `None` here and I don't know why ._.
    assert_eq!(spline.sample(1.0), None);

    // Any other value inside [0, 1] is fine:
    assert_eq!(spline.sample(0.123456789), Some(0.0));
}
```

[shady tools]: https://github.com/TornaxO7/shady
