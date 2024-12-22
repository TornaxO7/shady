use super::Uniform;

#[derive(Default, Debug, Clone, Copy)]
struct Coord {
    x: f32,
    y: f32,
}

pub struct Mouse {
    pressed: Coord,
    released: Coord,

    buffer: wgpu::Buffer,
    binding: u32,
}

impl Mouse {
    pub fn pressed_coord(&mut self, x: f32, y: f32) {
        self.pressed = Coord { x, y };
    }

    pub fn released_coord(&mut self, x: f32, y: f32) {
        self.released = Coord { x, y }
    }
}

impl Uniform for Mouse {
    type BufferDataType = [f32; 4];

    fn new(device: &wgpu::Device, binding: u32) -> Self {
        let buffer = Self::create_buffer(device);

        Self {
            pressed: Coord::default(),
            released: Coord::default(),

            binding,
            buffer,
        }
    }

    fn binding(&self) -> u32 {
        self.binding
    }

    fn buffer_label() -> &'static str {
        "Shady iMouse buffer"
    }

    fn update_buffer(&self, queue: &mut wgpu::Queue) {
        let data = [
            self.pressed.x,
            self.pressed.y,
            self.released.x,
            self.released.y,
        ];

        queue.write_buffer(self.buffer(), 0, bytemuck::cast_slice(&data));
    }

    fn cleanup(&mut self) {}

    fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}
