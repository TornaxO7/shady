use std::fmt;

use tracing::{debug, instrument};

use crate::template::TemplateGenerator;

use super::Resource;

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

impl Resource for Mouse {
    type BufferDataType = [f32; 4];

    fn new(device: &wgpu::Device) -> Self {
        let buffer = Self::create_uniform_buffer(device);

        Self {
            pos: Coord::default(),
            prev_state: MouseState::Released,
            prev_pos: Coord::default(),
            curr_pos: Coord::default(),

            buffer,
        }
    }

    fn binding() -> u32 {
        super::BindingValue::Mouse as u32
    }

    fn buffer_label() -> &'static str {
        "Shady iMouse buffer"
    }

    fn buffer_type() -> wgpu::BufferBindingType {
        wgpu::BufferBindingType::Uniform
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

    fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}

impl TemplateGenerator for Mouse {
    fn write_wgsl_template(
        writer: &mut dyn std::fmt::Write,
        bind_group_index: u32,
    ) -> Result<(), fmt::Error> {
        writer.write_fmt(format_args!(
            "
// x: x-coord when the mouse is pressed
// y: y-coord when the mouse is pressed
// z: x-coord when the mouse is released
// w: y-coord when the mouse is released
@group({}) @binding({})
var<uniform> iMouse: vec4<f32>;
",
            bind_group_index,
            Self::binding()
        ))
    }

    fn write_glsl_template(writer: &mut dyn fmt::Write) -> Result<(), fmt::Error> {
        writer.write_fmt(format_args!(
            "
// x: x-coord when the mouse is pressed
// y: y-coord when the mouse is pressed
// z: x-coord when the mouse is released
// w: y-coord when the mouse is released
layout(binding = {}) uniform vec4 iMouse;
",
            Self::binding()
        ))
    }
}
