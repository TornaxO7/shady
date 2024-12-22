use std::time::Instant;

use wgpu::Device;

use super::Uniform;

#[derive(Debug)]
pub struct Time {
    time: Instant,

    buffer: wgpu::Buffer,
    binding: u32,
}

impl Uniform for Time {
    type BufferDataType = f32;

    fn new(device: &Device, binding: u32) -> Self {
        let buffer = Self::create_buffer(device);

        Self {
            time: Instant::now(),
            buffer,
            binding,
        }
    }

    fn buffer_label() -> &'static str {
        "Shady iTime buffer"
    }

    fn binding(&self) -> u32 {
        self.binding
    }

    fn update_buffer(&self, queue: &mut wgpu::Queue) {
        let elapsed_time = self.time.elapsed().as_secs_f32();

        queue.write_buffer(self.buffer(), 0, bytemuck::cast_slice(&[elapsed_time]));
    }

    fn cleanup(&mut self) {}

    fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}
