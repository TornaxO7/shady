use realfft::num_complex::Complex32;
const AMOUNT_HIGHEST_MAGNITUDES: usize = 4;
const GRAVITY_VAL: f32 = 0.95;

const _: () = const {
    assert!(
        AMOUNT_HIGHEST_MAGNITUDES % 2 == 0,
        "For faster checking if the index in AvgRingBuffer is within the array-length."
    );
};

#[derive(Debug)]
pub struct Magnitudes {}

impl Magnitudes {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update_magnitudes(&mut self, fft_output: &[Complex32]) -> &[f32] {
        self.process_fft_output(fft_output);
        self.normalize_magnitudes();

        // returning the latest, would result into a "jump" for the bars for example
        self.buffers.prev()
    }

    pub fn update_with_ease(&mut self, ease_time: f32) -> &[f32] {
        debug_assert!(
            (0.0..=1.).contains(&ease_time),
            "`ease_time` ({}) must be between 0 and 1",
            ease_time
        );

        let t = simple_easing::sine_in_out(ease_time);
        let prev = self.buffers.prev();
        let curr = self.buffers.curr();

        for (i, mag) in self.magnitude_out.iter_mut().enumerate() {
            *mag = ((1. - t) * prev[i] + t * curr[i]) * GRAVITY_VAL;
        }

        self.magnitude_out.as_ref()
    }

    fn process_fft_output(&mut self, fft_output: &[Complex32]) {
        let prev_buffer = self.buffers.prev_mut();

        let mut max = f32::MIN;
        for (i, val) in fft_output.iter().enumerate() {
            let mag = val.norm();
            prev_buffer[i] = mag;

            if (START_FREQ..=END_FREQ).contains(&i) && mag > max {
                max = mag;
            }
        }

        self.highest_magnitudes.push(max);
        self.buffers.switch_current();
    }

    fn normalize_magnitudes(&mut self) {
        let max = self.current_highest_magnitude();
        let mag_buffer = self.buffers.curr_mut();

        if max < 1. {
            for mag in mag_buffer {
                *mag *= max;
            }
        } else {
            for mag in mag_buffer {
                *mag /= max;
            }
        }
    }

    fn current_highest_magnitude(&self) -> f32 {
        self.highest_magnitudes.avg()
    }
}

#[derive(Debug)]
struct DoubleBuffer<T> {
    buffer: Box<[T]>,

    capacity: usize,
    buf1_is_current: bool,
}

impl<T> DoubleBuffer<T> {
    pub fn new(val: T, capacity: usize) -> Self
    where
        T: Copy,
    {
        let buffer = vec![val; capacity * 2].into_boxed_slice();

        Self {
            buffer,
            capacity,
            buf1_is_current: true,
        }
    }

    pub fn switch_current(&mut self) {
        self.buf1_is_current = !self.buf1_is_current;
    }

    pub fn curr(&self) -> &[T] {
        let (start, end) = self.start_and_end(self.buf1_is_current);
        &self.buffer[start..end]
    }

    pub fn prev(&self) -> &[T] {
        let (start, end) = self.start_and_end(!self.buf1_is_current);
        &self.buffer[start..end]
    }

    pub fn curr_mut(&mut self) -> &mut [T] {
        let (start, end) = self.start_and_end(self.buf1_is_current);
        &mut self.buffer[start..end]
    }

    pub fn prev_mut(&mut self) -> &mut [T] {
        let (start, end) = self.start_and_end(!self.buf1_is_current);
        &mut self.buffer[start..end]
    }

    fn start_and_end(&self, use_buf1: bool) -> (usize, usize) {
        let offset = use_buf1 as usize;
        let start = self.capacity * offset;
        let end = self.capacity * (1 + offset);

        (start, end)
    }
}

#[derive(Debug, Clone)]
pub struct AvgRingBuffer {
    inner: Box<[f32]>,
    index: usize,
}

impl AvgRingBuffer {
    pub fn new() -> Self {
        Self {
            inner: vec![0.; AMOUNT_HIGHEST_MAGNITUDES].into_boxed_slice(),
            index: 0,
        }
    }

    pub fn push(&mut self, val: f32) {
        self.inner[self.index] = val;

        self.index = (self.index + 1) & !(1 << (AMOUNT_HIGHEST_MAGNITUDES / 2));
    }

    pub fn avg(&self) -> f32 {
        self.inner.iter().sum::<f32>() / self.inner.len() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod avg_ring_buffer {
        use super::*;

        #[test]
        fn add_to_full_array() {
            let mut buffer = AvgRingBuffer::new();

            for i in 0..AMOUNT_HIGHEST_MAGNITUDES {
                buffer.push(i as f32);
            }

            assert_eq!(
                *buffer.inner.last().unwrap() as usize,
                AMOUNT_HIGHEST_MAGNITUDES - 1
            );

            assert_eq!(
                buffer.index, 0,
                "Index did not went to the begininng. Index is at: {}",
                buffer.index
            );
        }
    }
}
