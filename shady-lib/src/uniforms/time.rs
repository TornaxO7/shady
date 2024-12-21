use std::time::Instant;

use super::Uniform;

#[derive(Debug)]
pub struct Time(Instant);

impl Default for Time {
    fn default() -> Self {
        Self(Instant::now())
    }
}

impl Uniform for Time {
    type BufferDataType = f32;

    fn buffer_label() -> &'static str {
        "Shady iTime buffer"
    }

    fn binding() -> u32 {
        0
    }

    fn update_buffer(&self, queue: &mut wgpu::Queue, device: &wgpu::Device) {
        let elapsed_time = self.0.elapsed().as_secs_f32();

        queue.write_buffer(&Self::buffer(device), 0, &elapsed_time.to_ne_bytes());
    }

    fn cleanup(&mut self) {}
}
