use tracing::{debug, instrument};

use super::Uniform;

#[derive(Default, Debug, Clone, Copy)]
struct Coord {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseState {
    Pressed,
    Released,
}

pub struct Mouse {
    pos: Coord,

    prev_state: MouseState,
    prev_pos: Coord,
    curr_pos: Coord,

    buffer: wgpu::Buffer,
    binding: u32,
}

impl Mouse {
    #[instrument(skip(self), level = "trace")]
    pub fn cursor_moved(&mut self, x: f32, y: f32) {
        self.pos = Coord { x, y };
    }

    #[instrument(skip(self), level = "trace")]
    pub fn mouse_input(&mut self, state: MouseState) {
        self.prev_state = state;
        if state == MouseState::Pressed {
            self.curr_pos = self.pos;
            debug!("Mouse curr pos: {:?}", self.curr_pos);

            if self.prev_state == MouseState::Released {
                self.prev_pos = self.pos;
                debug!("Mouse prev pos: {:?}", self.prev_pos);
            }
        } else {
            self.prev_pos = Coord::default();
        }
    }
}

impl Uniform for Mouse {
    type BufferDataType = [f32; 4];

    fn new(device: &wgpu::Device, binding: u32) -> Self {
        let buffer = Self::create_buffer(device);

        Self {
            pos: Coord::default(),
            prev_state: MouseState::Released,
            prev_pos: Coord::default(),
            curr_pos: Coord::default(),

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
            self.curr_pos.x,
            self.curr_pos.y,
            self.prev_pos.x,
            self.prev_pos.y,
        ];

        queue.write_buffer(self.buffer(), 0, bytemuck::cast_slice(&data));
    }

    fn cleanup(&mut self) {}

    fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}
