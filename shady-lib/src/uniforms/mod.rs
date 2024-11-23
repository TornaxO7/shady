mod audio;

use std::time::Instant;

use audio::AudioData;
use wgpu::{util::DeviceExt, Device};

pub struct Uniforms {
    pub data: UniformData,
    pub buffers: UniformBuffers,
    pub bind: UniformBind,
}

pub struct UniformData {
    pub i_time: Instant,
    pub i_resolution: [f32; 3],
    pub audio: AudioData,
}

#[derive(Debug)]
pub struct UniformBuffers {
    pub i_time: wgpu::Buffer,
    pub i_resolution: wgpu::Buffer,
    pub i_audio: wgpu::Buffer,
}

#[derive(Debug)]
pub struct UniformBind {
    pub layout: wgpu::BindGroupLayout,
    pub group: wgpu::BindGroup,
}

impl UniformData {
    pub fn update_resolution(&mut self, width: f32, height: f32) {
        self.i_resolution = [width, height, width / height];
    }
}

impl Default for UniformData {
    fn default() -> Self {
        Self {
            i_time: Instant::now(),
            i_resolution: [0.0; 3],
            audio: AudioData::new(),
        }
    }
}

impl Uniforms {
    pub fn new(device: &Device) -> Self {
        let data = UniformData::default();
        let buffers = UniformBuffers {
            i_time: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("iTime uniform buffer"),
                contents: bytemuck::cast_slice(&[data.i_time.elapsed().as_secs_f32()]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
            i_resolution: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("iResolution uniform buffer"),
                contents: bytemuck::cast_slice(&data.i_resolution),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
            i_audio: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("iAudio uniform buffer"),
                contents: bytemuck::cast_slice(&data.audio.data()),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
        };

        let bind = {
            let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Bind group"),
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffers.i_time.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: buffers.i_resolution.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: buffers.i_audio.as_entire_binding(),
                    },
                ],
            });

            UniformBind { layout, group }
        };

        Self {
            data,
            buffers,
            bind,
        }
    }
}
