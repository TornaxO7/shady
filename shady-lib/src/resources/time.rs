use std::{fmt, time::Instant};

use wgpu::Device;

use crate::template::TemplateGenerator;

use super::Resource;

#[derive(Debug)]
pub struct Time {
    time: Instant,

    buffer: wgpu::Buffer,
}

impl Resource for Time {
    fn new(device: &Device) -> Self {
        let buffer = Self::create_uniform_buffer(device, std::mem::size_of::<f32>() as u64);

        Self {
            time: Instant::now(),
            buffer,
        }
    }

    fn buffer_label() -> &'static str {
        "Shady iTime buffer"
    }

    fn buffer_type() -> wgpu::BufferBindingType {
        wgpu::BufferBindingType::Uniform
    }

    fn binding() -> u32 {
        super::BindingValue::Time as u32
    }

    fn update_buffer(&self, queue: &wgpu::Queue) {
        let elapsed_time = self.time.elapsed().as_secs_f32();

        queue.write_buffer(self.buffer(), 0, bytemuck::cast_slice(&[elapsed_time]));
    }

    fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}

impl TemplateGenerator for Time {
    fn write_wgsl_template(
        writer: &mut dyn std::fmt::Write,
        bind_group_index: u32,
    ) -> Result<(), fmt::Error> {
        writer.write_fmt(format_args!(
            "
@group({}) @binding({})
var<uniform> iTime: f32;
",
            bind_group_index,
            Self::binding()
        ))
    }

    fn write_glsl_template(writer: &mut dyn fmt::Write) -> Result<(), fmt::Error> {
        writer.write_fmt(format_args!(
            "
layout(binding = {}) uniform float iTime;
",
            Self::binding()
        ))
    }
}
