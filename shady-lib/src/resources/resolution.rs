use wgpu::Device;

use super::Resource;

#[derive(Debug)]
pub struct Resolution {
    width: u32,
    height: u32,

    buffer: wgpu::Buffer,
    binding: u32,
}

impl Resolution {
    pub fn update_resolution(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.width = width;
            self.height = height;
        }
    }
}

impl Resource for Resolution {
    type BufferDataType = [f32; 2];

    fn new(device: &Device, binding: u32) -> Self {
        let buffer = Self::create_uniform_buffer(device);

        Self {
            width: 0,
            height: 0,
            buffer,
            binding,
        }
    }

    fn buffer_label() -> &'static str {
        "Shady iResolution buffer"
    }

    fn buffer_type() -> wgpu::BufferBindingType {
        wgpu::BufferBindingType::Uniform
    }

    fn binding(&self) -> u32 {
        self.binding
    }

    fn update_buffer(&self, queue: &mut wgpu::Queue) {
        let data = {
            let width = self.width as f32;
            let height = self.height as f32;

            [width, height]
        };

        queue.write_buffer(self.buffer(), 0, bytemuck::cast_slice(&data));
    }

    fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}
