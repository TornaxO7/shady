use std::fmt;

use crate::{template::TemplateGenerator, ShadyDescriptor};

use super::Resource;

#[derive(Debug)]
pub struct Resolution {
    width: u32,
    height: u32,

    buffer: wgpu::Buffer,
}

impl Resolution {
    pub fn set(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.width = width;
            self.height = height;
        }
    }
}

impl Resource for Resolution {
    fn new(desc: &ShadyDescriptor) -> Self {
        let buffer =
            Self::create_uniform_buffer(desc.device, std::mem::size_of::<[f32; 2]>() as u64);

        Self {
            width: 0,
            height: 0,
            buffer,
        }
    }

    fn buffer_label() -> &'static str {
        "Shady iResolution buffer"
    }

    fn buffer_type() -> wgpu::BufferBindingType {
        wgpu::BufferBindingType::Uniform
    }

    fn binding() -> u32 {
        super::BindingValue::Resolution as u32
    }

    fn update_buffer(&self, queue: &wgpu::Queue) {
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

impl TemplateGenerator for Resolution {
    fn write_wgsl_template(
        writer: &mut dyn std::fmt::Write,
        bind_group_index: u32,
    ) -> Result<(), fmt::Error> {
        writer.write_fmt(format_args!(
            "
// x: width
// y: height
@group({}) @binding({})
var<uniform> iResolution: vec2<f32>;
",
            bind_group_index,
            Self::binding()
        ))
    }

    fn write_glsl_template(writer: &mut dyn fmt::Write) -> Result<(), fmt::Error> {
        writer.write_fmt(format_args!(
            "
// x: width
// y: height
layout(binding = {}) uniform vec2 iResolution;
",
            Self::binding()
        ))
    }
}
