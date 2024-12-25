use realfft::num_complex::Complex32;

type Hz = usize;
pub type Fraction = f32;

const START_MAX_FREQ: Hz = 20;
// 20–60 Hz
// Deep bass sounds (kick drums, low rumbles). Ideal for big, bold movements.
const SUB_BASS_FRACTION: Fraction =
    ((60 - 20) / 2 + 20) as Fraction / BRILLIANCE_MAX_FREQ as Fraction;
const SUB_BASS_MAX_FREQ: Hz = 60;
// 60–250 Hz
// Regular bass (bass guitars, drums, lower electronic beats).
const BASS_FRACTION: Fraction = ((250 - 60) / 2 + 60) as Fraction / BRILLIANCE_MAX_FREQ as Fraction;
const BASS_MAX_FREQ: Hz = 250;
// 250–500 Hz
// Lower midrange, covering warm tones (vocals, chords, rhythm).
const LOW_MIDRANGE_FRACTION: Fraction =
    ((500 - 250) / 2 + 250) as Fraction / BRILLIANCE_MAX_FREQ as Fraction;
const LOW_MIDRANGE_MAX_FREQ: Hz = 500;
// 500–2000 Hz
// Core of the mix: main vocals, instruments like guitars, snares.
const MIDRANGE_FRACTION: Fraction =
    ((2000 - 500) / 2 + 500) as Fraction / BRILLIANCE_MAX_FREQ as Fraction;
const MIDRANGE_MAX_FREQ: Hz = 2000;
// 2000–4000 Hz
// Higher detail in vocals, clarity of sounds (synths, lead melodies).
const UPPER_MIDRANGE_FRACTION: Fraction =
    ((4000 - 2000) / 2 + 2_000) as Fraction / BRILLIANCE_MAX_FREQ as Fraction;
const UPPER_MIDRANGE_MAX_FREQ: Hz = 4_000;
// 4000–6000 Hz
// Contributes to sound clarity and presence (hi-hats, sharp transients).
const PRESENCE_FRACTION: Fraction =
    ((6000 - 4000) / 2 + 4_000) as Fraction / BRILLIANCE_MAX_FREQ as Fraction;
const PRESENCE_MAX_FREQ: Hz = 6_000;
// 6000–20000 Hz
// High-pitched sounds and airiness (cymbals, shimmering effects).
const BRILLIANCE_FRACTION: Fraction =
    ((20_000 - 6_000) / 2 + 6_000) as Fraction / BRILLIANCE_MAX_FREQ as Fraction;
pub const BRILLIANCE_MAX_FREQ: Hz = 20_000;

// Range is from 0 to 20_000
pub const FRACTIONS: [Fraction; AMOUNT_ENTRIES] = [
    SUB_BASS_FRACTION,
    BASS_FRACTION,
    LOW_MIDRANGE_FRACTION,
    MIDRANGE_FRACTION,
    UPPER_MIDRANGE_FRACTION,
    PRESENCE_FRACTION,
    BRILLIANCE_FRACTION,
];

pub const SUB_BASS: usize = 0;
pub const BASS: usize = 1;
pub const LOW_MIDRANGE: usize = 2;
pub const MIDRANGE: usize = 3;
pub const UPPER_MIDRANGE: usize = 4;
pub const PRESENCE: usize = 5;
pub const BRILLIANCE: usize = 6;
pub const AMOUNT_ENTRIES: usize = BRILLIANCE + 1;

/// Includes (from left to right):
///     - sub bass
///     - bass
///     - low_midrange
///     - etc.
#[derive(Default, Clone, Debug)]
pub struct FrequencyBandValues(pub [f32; AMOUNT_ENTRIES]);

impl FrequencyBandValues {
    pub fn new(fft_output: &[Complex32], frequency_step: usize) -> Self {
        let range_steps: [Hz; AMOUNT_ENTRIES] = [
            SUB_BASS_MAX_FREQ / frequency_step,
            BASS_MAX_FREQ / frequency_step,
            LOW_MIDRANGE_MAX_FREQ / frequency_step,
            MIDRANGE_MAX_FREQ / frequency_step,
            UPPER_MIDRANGE_MAX_FREQ / frequency_step,
            PRESENCE_MAX_FREQ / frequency_step,
            BRILLIANCE_MAX_FREQ / frequency_step,
        ];

        let mut values = [0.0; AMOUNT_ENTRIES];

        let mut start = START_MAX_FREQ / frequency_step;
        for (index, &end) in range_steps.iter().enumerate() {
            values[index] = fft_output[start..end]
                .iter()
                .map(|value| value.norm())
                .sum::<f32>()
                / (end - start) as f32;

            start = end;
        }

        Self(values)
    }

    pub fn max_freq(&self) -> f32 {
        let mut max = self.0[0];
        for &value in self.0.iter() {
            if max < value {
                max = value;
            }
        }
        max
    }

    pub fn normalize(&mut self, max_value: f32) {
        for value in self.0.iter_mut() {
            *value /= max_value;
        }
    }
}
