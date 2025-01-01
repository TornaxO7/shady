use super::Resource;

pub struct Frame {
    value: u32,

    binding: u32,
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

impl Resource for Frame {
    type BufferDataType = u32;

    fn new(device: &wgpu::Device, binding: u32) -> Self {
        let buffer = Self::create_uniform_buffer(device);

        Self {
            value: 0,
            buffer,
            binding,
        }
    }

    fn binding(&self) -> u32 {
        self.binding
    }

    fn buffer_label() -> &'static str {
        "Shady iFrame buffer"
    }

    fn buffer_type() -> wgpu::BufferBindingType {
        wgpu::BufferBindingType::Uniform
    }

    fn update_buffer(&self, queue: &mut wgpu::Queue) {
        queue.write_buffer(self.buffer(), 0, &self.value.to_ne_bytes());
    }

    fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}