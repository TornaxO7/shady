use realfft::{
    num_complex::{Complex, Complex32},
    FftNum, RealFftPlanner,
};

pub struct FftCalculator {
    planner: RealFftPlanner<f32>,
    scratch_buffer: Vec<Complex32>,
    fft_output: Vec<Complex32>,
    fft_size: usize,
    magnitudes: Vec<f32>,

    highest_magnitude: f32,
}

impl FftCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn process(&mut self, data: &mut Vec<f32>) -> (usize, &[f32]) {
        assert!(!data.is_empty());

        self.calc_fft(data);
        self.calc_magnitudes();
        // self.adjust_magnitudes();
        self.update_max_min_magnitude();
        self.normalize_magnitudes();

        (self.fft_size(), self.magnitudes())
    }

    pub fn magnitudes(&self) -> &[f32] {
        &self.magnitudes
    }

    pub fn fft_size(&self) -> usize {
        self.fft_size
    }

    fn update_max_min_magnitude(&mut self) {
        for &val in self.magnitudes.iter() {
            // if val < self.lowest_magnitude {
            //     self.lowest_magnitude = val;
            // }
            if val > self.highest_magnitude {
                self.highest_magnitude = val;
            }
        }
    }

    fn calc_fft(&mut self, data: &mut Vec<f32>) {
        if data.len() % 2 != 0 {
            data.push(0.);
        }

        self.fft_size = data.len();
        let fft = self.planner.plan_fft_forward(self.fft_size);
        self.fft_output
            .resize(self.fft_size / 2 + 1, Complex32::ZERO);
        self.scratch_buffer
            .resize(fft.get_scratch_len(), Complex32::ZERO);

        fft.process_with_scratch(
            data,
            self.fft_output.as_mut_slice(),
            self.scratch_buffer.as_mut_slice(),
        )
        .unwrap();
    }

    // Calculates the magnitudes out of the fft output
    fn calc_magnitudes(&mut self) {
        self.magnitudes.resize(self.fft_output.len(), 0.);
        for (i, val) in self.fft_output.iter().enumerate() {
            self.magnitudes[i] = val.norm();
        }
    }

    // to make higher frequencies louder
    fn adjust_magnitudes(&mut self) {
        let mag_len = self.magnitudes.len();
        for (i, val) in self.magnitudes.iter_mut().enumerate() {
            let percentage = (i + 1) as f32 / mag_len as f32;

            let log: f32 = 1. / 2f32.log(percentage + 1.);
            let exp: f32 = percentage.sqrt();

            *val *= (log + exp) / 2. * 0.1;
        }
    }

    fn normalize_magnitudes(&mut self) {
        for mag in self.magnitudes.iter_mut() {
            *mag /= self.highest_magnitude;
        }
    }
}

impl Default for FftCalculator {
    fn default() -> Self {
        Self {
            planner: RealFftPlanner::new(),
            scratch_buffer: Vec::new(),
            fft_output: Vec::new(),
            magnitudes: Vec::new(),
            fft_size: 0,
            highest_magnitude: f32::MIN,
        }
    }
}
