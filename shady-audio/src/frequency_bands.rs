type Hz = usize;

/// The fraction sets the index from value within [0; 1]
pub type Fraction = f32;

const START_MAX_FREQ: Hz = 20;
const START_FRACTION: Fraction = 0.0;

// 20–60 Hz
// Deep bass sounds (kick drums, low rumbles). Ideal for big, bold movements.
const SUB_BASS_FRACTION: Fraction = (SUB_BASS_MAX_FREQ - START_MAX_FREQ) as Fraction
    / (BRILLIANCE_MAX_FREQ - START_MAX_FREQ) as Fraction;
const SUB_BASS_MAX_FREQ: Hz = 60;
// 60–250 Hz
// Regular bass (bass guitars, drums, lower electronic beats).
const BASS_FRACTION: Fraction = (BASS_MAX_FREQ - START_MAX_FREQ) as Fraction
    / (BRILLIANCE_MAX_FREQ - START_MAX_FREQ) as Fraction;
const BASS_MAX_FREQ: Hz = 250;
// 250–500 Hz
// Lower midrange, covering warm tones (vocals, chords, rhythm).
const LOW_MIDRANGE_FRACTION: Fraction = (LOW_MIDRANGE_MAX_FREQ - START_MAX_FREQ) as Fraction
    / (BRILLIANCE_MAX_FREQ - START_MAX_FREQ) as Fraction;
const LOW_MIDRANGE_MAX_FREQ: Hz = 500;
// 500–2000 Hz
// Core of the mix: main vocals, instruments like guitars, snares.
const MIDRANGE_FRACTION: Fraction = (MIDRANGE_MAX_FREQ - START_MAX_FREQ) as Fraction
    / (BRILLIANCE_MAX_FREQ - START_MAX_FREQ) as Fraction;
const MIDRANGE_MAX_FREQ: Hz = 2000;
// 2000–4000 Hz
// Higher detail in vocals, clarity of sounds (synths, lead melodies).
const UPPER_MIDRANGE_FRACTION: Fraction = (UPPER_MIDRANGE_MAX_FREQ - START_MAX_FREQ) as Fraction
    / (BRILLIANCE_MAX_FREQ - START_MAX_FREQ) as Fraction;
const UPPER_MIDRANGE_MAX_FREQ: Hz = 4_000;
// 4000–6000 Hz
// Contributes to sound clarity and presence (hi-hats, sharp transients).
const PRESENCE_FRACTION: Fraction = (PRESENCE_MAX_FREQ - START_MAX_FREQ) as Fraction
    / (BRILLIANCE_MAX_FREQ - START_MAX_FREQ) as Fraction;
const PRESENCE_MAX_FREQ: Hz = 6_000;
// 6000–20000 Hz
// High-pitched sounds and airiness (cymbals, shimmering effects).
const BRILLIANCE_FRACTION: Fraction = (BRILLIANCE_MAX_FREQ - START_MAX_FREQ) as Fraction
    / (BRILLIANCE_MAX_FREQ - START_MAX_FREQ) as Fraction;
pub const BRILLIANCE_MAX_FREQ: Hz = 20_000;

// Range is from 0 to 20_000
pub const FRACTIONS: [Fraction; AMOUNT_ENTRIES] = [
    START_FRACTION,
    SUB_BASS_FRACTION,
    BASS_FRACTION,
    LOW_MIDRANGE_FRACTION,
    MIDRANGE_FRACTION,
    UPPER_MIDRANGE_FRACTION,
    PRESENCE_FRACTION,
    BRILLIANCE_FRACTION,
];

pub const START: usize = 0;
pub const SUB_BASS: usize = START + 1;
pub const BASS: usize = SUB_BASS + 1;
pub const LOW_MIDRANGE: usize = BASS + 1;
pub const MIDRANGE: usize = LOW_MIDRANGE + 1;
pub const UPPER_MIDRANGE: usize = MIDRANGE + 1;
pub const PRESENCE: usize = UPPER_MIDRANGE + 1;
pub const BRILLIANCE: usize = PRESENCE + 1;
pub const AMOUNT_ENTRIES: usize = BRILLIANCE + 1;

/// Includes (from left to right):
///     - sub bass
///     - bass
///     - low_midrange
///     - etc.
#[derive(Default, Clone, Debug)]
pub struct FrequencyBandValues(pub [f32; AMOUNT_ENTRIES]);

impl FrequencyBandValues {
    pub fn new(
        magnitudes: &[f32],
        frequency_step: usize,
        normalize_magnitude: impl Fn(f32) -> f32,
    ) -> Self {
        // Every magnitude from freq_indices[i] to freq_indices[i + 1]
        // belongs to the frequency class i.
        let freq_indices: [Hz; AMOUNT_ENTRIES] = [
            START_MAX_FREQ / frequency_step,
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
        for (index, &end) in freq_indices.iter().enumerate() {
            values[index] = normalize_magnitude(
                magnitudes[start..end].iter().sum::<f32>() / (end - start) as f32,
            );

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
