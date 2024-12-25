use std::num::NonZeroUsize;

use shady_audio::ShadyAudio;

fn main() {
    println!("Hello there");

    let mut audio = ShadyAudio::new(None, None, |err| panic!("{}", err));

    // get the magnitudes with 10 entries
    let magnitudes = audio.fetch_magnitudes_mut(NonZeroUsize::new(10).unwrap());
    assert_eq!(magnitudes.len(), 10);

    // ... or in normalized form
    let norm_magnitudes = audio.fetch_magnitudes_normalized(NonZeroUsize::new(10).unwrap());
    for &norm_magn in norm_magnitudes {
        assert!(0.0 <= norm_magn);
        assert!(norm_magn <= 1.0);
    }
    assert_eq!(norm_magnitudes.len(), 10);
}
