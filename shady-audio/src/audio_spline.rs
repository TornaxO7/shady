use crate::{fft, START_FREQ};
use splines::{Key, Spline};

const EXP_BASE: f32 = 1.06;

pub struct FreqSpline {
    spline: Spline<f32, f32>,
}

impl FreqSpline {
    pub fn new() -> Self {
        let amount_points = {
            let dummy_magnitudes = [0.; fft::FFT_OUTPUT_SIZE];
            MagnitudeIterator::new(&dummy_magnitudes).count()
        };

        let spline = {
            let mut spline = Spline::from_vec(Vec::with_capacity(amount_points));

            let step = 1. / (amount_points - 1) as f32; // `-1` in order to reach `1.`

            for i in 0..amount_points as usize {
                let x = i as f32 * step;
                let key = Key::new(x, 0.0, splines::Interpolation::Linear);
                spline.add(key);
            }
            spline
        };

        #[cfg(debug_assertions)]
        {
            let keys = spline.keys();

            check_equidistance(keys);
            check_1_0_point_exists(keys);
        }

        Self { spline }
    }

    pub fn update(&mut self, magnitudes: &[f32]) {
        debug_assert_eq!(magnitudes.len(), fft::FFT_OUTPUT_SIZE);

        let iterator = MagnitudeIterator::new(magnitudes);

        for (i, value) in iterator.enumerate() {
            *self.spline.get_mut(i as usize).unwrap().value = value;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.spline.is_empty()
    }

    pub fn sample(&self, t: f32) -> Option<f32> {
        self.spline.sample(t)
    }
}

struct MagnitudeIterator<'a> {
    magnitudes: &'a [f32],

    last_entry_calculated: bool,
    offset: i32,
}

impl<'a> MagnitudeIterator<'a> {
    pub fn new(magnitudes: &'a [f32]) -> Self {
        Self {
            magnitudes,
            offset: 0,
            last_entry_calculated: false,
        }
    }
}

impl<'a> Iterator for MagnitudeIterator<'a> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.offset += 1;
        let prev = (START_FREQ as f32 * EXP_BASE.powi(self.offset - 1)) as usize;
        let next = (prev as f32 * EXP_BASE) as usize;

        if next > self.magnitudes.len() {
            if self.last_entry_calculated {
                None
            } else {
                self.last_entry_calculated = true;
                Some(self.magnitudes[prev..].iter().sum::<f32>() / (next - prev) as f32)
            }
        } else {
            Some(self.magnitudes[prev..next].iter().sum::<f32>() / (next - prev) as f32)
        }
    }
}

fn check_equidistance(keys: &[Key<f32, f32>]) {
    for (i, group) in keys.chunks_exact(3).enumerate() {
        let distance_is_same = {
            let right_chunk = group[2].t - group[1].t;
            let left_chunk = group[1].t - group[0].t;

            (right_chunk - left_chunk).abs() < 0.000001
        };

        debug_assert!(
            distance_is_same,
            "Spline points are not equidistant starting from: {}. Keys:\n{:#?}",
            i, keys
        );
    }
}

fn check_1_0_point_exists(keys: &[Key<f32, f32>]) {
    let last_key = keys.last().unwrap();

    debug_assert!(
        (1.0 - last_key.t) < 0.00001,
        "Missing the last point at t = 1.0 of the spline. Keys:\n{:#?}",
        keys
    );
}
