use super::Uniform;

pub struct Frame {
    value: u32,

    buffer: wgpu::Buffer,
}

impl Frame {
    pub fn next_frame(&mut self) {
        self.value += 1;
    }

    pub fn reset_counter(&mut self) {
        self.value = 0;
    }
}

impl Uniform for Frame {
    type BufferDataType = u32;

    fn new(device: &wgpu::Device) -> Self {
        let buffer = Self::create_buffer(device);

        Self { value: 0, buffer }
    }

    fn binding() -> u32 {
        4
    }

    fn buffer_label() -> &'static str {
        "Shady iFrame buffer"
    }

    fn update_buffer(&self, queue: &mut wgpu::Queue) {
        queue.write_buffer(self.buffer(), 0, &self.value.to_ne_bytes());
    }

    fn cleanup(&mut self) {}

    fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}
