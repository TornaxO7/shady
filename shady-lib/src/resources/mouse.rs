use std::fmt;

use tracing::instrument;

use crate::template::TemplateGenerator;

use super::Resource;

const DESC: &str = "\
// xy (index 0 and 1): The xy coordinate of the mouse while the user holds the left button
// zw (index 2 and 3): The xy coordinate of the mouse where the user starts holding the left button";

#[derive(Default, Debug, Clone, Copy)]
struct Coord {
    pub x: f32,
    pub y: f32,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseState {
    Released,
    Pressed,
}

pub struct Mouse {
    pos: Coord,

    prev_state: MouseState,
    curr_state: MouseState,
    pressed_pos: Coord,
    first_click_coord: Coord,

    buffer: wgpu::Buffer,
}

impl Mouse {
    #[instrument(skip(self), level = "trace")]
    pub fn cursor_moved(&mut self, x: f32, y: f32) {
        self.pos = Coord { x, y };

        if self.curr_state == MouseState::Pressed {
            self.pressed_pos = self.pos;
        }
    }

    #[instrument(skip(self), level = "trace")]
    pub fn mouse_input(&mut self, state: MouseState) {
        if self.curr_state == MouseState::Pressed && self.prev_state == MouseState::Released {
            self.first_click_coord = self.pos;
        }

        self.prev_state = self.curr_state;
        self.curr_state = state;
    }
}

impl Resource for Mouse {
    type BufferDataType = [f32; 4];

    fn new(device: &wgpu::Device) -> Self {
        let buffer = Self::create_uniform_buffer(device);

        Self {
            pos: Coord::default(),
            first_click_coord: Coord::default(),
            pressed_pos: Coord::default(),

            prev_state: MouseState::Released,
            curr_state: MouseState::Released,

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
            self.pressed_pos.x,
            self.pressed_pos.y,
            self.first_click_coord.x,
            self.first_click_coord.y,
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
{}
@group({}) @binding({})
var<uniform> iMouse: vec4<f32>;
",
            DESC,
            bind_group_index,
            Self::binding()
        ))
    }

    fn write_glsl_template(writer: &mut dyn fmt::Write) -> Result<(), fmt::Error> {
        writer.write_fmt(format_args!(
            "
{}
layout(binding = {}) uniform vec4 iMouse;
",
            DESC,
            Self::binding()
        ))
    }
}
