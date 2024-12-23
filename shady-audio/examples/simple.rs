use std::num::NonZeroUsize;

use cpal::traits::HostTrait;
use shady_audio::ShadyAudio;

fn main() {
    println!("Hello there");

    let host = cpal::default_host();

    let device = host.default_output_device().unwrap();

    let mut audio = ShadyAudio::new(&device, None, |err| panic!("{}", err));

    // get the magnitudes with 10 entries
    let _ = audio.fetch_magnitudes(NonZeroUsize::new(10).unwrap());

    // ... or in normalized form
    let _ = audio.fetch_magnitudes_normalized(NonZeroUsize::new(10).unwrap());
}
