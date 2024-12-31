use realfft::num_complex::Complex32;
use ringbuffer::{AllocRingBuffer, RingBuffer};

use crate::{END_FREQ, START_FREQ};

const BUFFER_SIZE: usize = crate::fft::FFT_OUTPUT_SIZE;
const AMOUNT_HIGHEST_MAGNITUDES: usize = 4;

#[derive(Debug)]
pub struct Magnitudes {
    highest_magnitudes: AllocRingBuffer<f32>,

    buffers: DoubleBuffer<f32>,

    magnitude_out: Box<[f32; BUFFER_SIZE]>,
}

impl Magnitudes {
    pub fn new() -> Self {
        let highest_magnitudes = AllocRingBuffer::new(AMOUNT_HIGHEST_MAGNITUDES);

        let magnitude_out = Box::new([0.; BUFFER_SIZE]);
        let buffers = DoubleBuffer::new(0., BUFFER_SIZE);

        Self {
            highest_magnitudes,
            buffers,
            magnitude_out,
        }
    }

    pub fn update_magnitudes(&mut self, fft_output: &[Complex32]) -> &[f32] {
        self.process_fft_output(fft_output);
        self.normalize_magnitudes();

        // returning the latest, would result into a "jump" for the bars for example
        self.buffers.prev()
    }

    pub fn update_with_ease(&mut self, ease_time: f32) -> &[f32] {
        debug_assert!(
            0.0 <= ease_time && ease_time <= 1.,
            "`ease_time` ({}) must be between 0 and 1",
            ease_time
        );

        let t = simple_easing::sine_in_out(ease_time);
        let prev = self.buffers.prev();
        let curr = self.buffers.curr();

        for (i, mag) in self.magnitude_out.iter_mut().enumerate() {
            *mag = (1. - t) * prev[i] + t * curr[i];
        }

        self.magnitude_out.as_ref()
    }

    fn process_fft_output(&mut self, fft_output: &[Complex32]) {
        let prev_buffer = self.buffers.prev_mut();

        let mut max = f32::MIN;
        for (i, val) in fft_output.iter().enumerate() {
            let mag = val.norm();
            prev_buffer[i] = mag;

            if START_FREQ <= i && i <= END_FREQ {
                if mag > max {
                    max = mag;
                }
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
        self.highest_magnitudes.iter().sum::<f32>() / AMOUNT_HIGHEST_MAGNITUDES as f32 * 1.25
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
        let offset = self.buf1_is_current as usize;
        let start = self.capacity * offset;
        let end = self.capacity * (1 + offset);

        &self.buffer[start..end]
    }

    pub fn prev(&self) -> &[T] {
        let offset = !self.buf1_is_current as usize;
        let start = self.capacity * offset;
        let end = self.capacity * (1 + offset);

        &self.buffer[start..end]
    }

    pub fn curr_mut(&mut self) -> &mut [T] {
        let offset = self.buf1_is_current as usize;
        let start = self.capacity * offset;
        let end = self.capacity * (1 + offset);

        &mut self.buffer[start..end]
    }

    pub fn prev_mut(&mut self) -> &mut [T] {
        let offset = !self.buf1_is_current as usize;
        let start = self.capacity * offset;
        let end = self.capacity * (1 + offset);

        &mut self.buffer[start..end]
    }
}
