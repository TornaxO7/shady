use super::Uniform;

#[derive(Default, Debug)]
pub struct Resolution {
    width: u32,
    height: u32,
}

impl Resolution {
    pub fn update_resolution(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.width = width;
            self.height = height;
        }
    }
}

impl Uniform for Resolution {
    type BufferDataType = [f32; 2];

    fn buffer_label() -> &'static str {
        "Shady iResolution buffer"
    }

    fn binding() -> u32 {
        1
    }

    fn update_buffer(&self, queue: &mut wgpu::Queue, device: &wgpu::Device) {
        let data = {
            let width = self.width as f32;
            let height = self.height as f32;

            [width, height]
        };

        queue.write_buffer(&Self::buffer(device), 0, bytemuck::cast_slice(&data));
    }

    fn cleanup(&mut self) {}
}
