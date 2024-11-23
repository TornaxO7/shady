mod uniforms;
mod vertices;

use std::ops::Range;

use uniforms::Uniforms;
use wgpu::Device;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid fragment shader in line {line_num}, column {line_pos}: {msg}")]
    InvalidFragmentShader {
        fragment_code: String,
        msg: String,
        line_num: u32,
        line_pos: u32,
        offset: u32,
        length: u32,
    },
}

pub struct Shady {
    pub vbuffer: wgpu::Buffer,
    pub ibuffer: wgpu::Buffer,
    uniforms: Uniforms,
}

impl Shady {
    pub fn new(device: &Device) -> Self {
        Self {
            vbuffer: vertices::get_vertex_buffer(device),
            ibuffer: vertices::get_index_buffer(device),
            uniforms: Uniforms::new(device),
        }
    }

    pub fn ibuffer_range(&self) -> Range<u32> {
        0..vertices::INDICES.len() as u32
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.uniforms.bind.group
    }

    pub fn update_resolution(&mut self, width: f32, height: f32) {
        self.uniforms.data.update_resolution(width, height);
    }

    pub fn cleanup(&mut self) {
        self.uniforms.data.audio.cleanup();
    }

    pub fn update_buffers(&self, queue: &mut wgpu::Queue) {
        queue.write_buffer(
            &self.uniforms.buffers.i_time,
            0,
            bytemuck::cast_slice(&[self.uniforms.data.i_time.elapsed().as_secs_f32()]),
        );

        queue.write_buffer(
            &self.uniforms.buffers.i_resolution,
            0,
            bytemuck::cast_slice(&self.uniforms.data.i_resolution),
        );

        queue.write_buffer(
            &self.uniforms.buffers.i_audio,
            0,
            bytemuck::cast_slice(&self.uniforms.data.audio.data()),
        );
    }

    pub fn get_render_pipeline<S: AsRef<str>>(
        &self,
        device: &Device,
        fragment_shader: S,
        texture_format: &wgpu::TextureFormat,
    ) -> Result<wgpu::RenderPipeline, Error> {
        let vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shady vertex shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        // check if the fragment is valid
        if let Err(err) = wgpu::naga::front::wgsl::parse_str(fragment_shader.as_ref()) {
            let msg = err.message().to_string();
            let location = err.location(fragment_shader.as_ref()).unwrap();

            return Err(Error::InvalidFragmentShader {
                msg,
                fragment_code: fragment_shader.as_ref().to_string(),
                line_num: location.line_number,
                line_pos: location.line_position,
                offset: location.offset,
                length: location.length,
            });
        }
        let fragment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shady fragment shader"),
            source: wgpu::ShaderSource::Wgsl(fragment_shader.as_ref().into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Shady pipeline layout"),
            bind_group_layouts: &[&self.uniforms.bind.layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Shady render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vertex_shader,
                entry_point: "vertex_main",
                buffers: &[vertices::BUFFER_LAYOUT],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &fragment_shader,
                entry_point: "main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: *texture_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview: None,
            cache: None,
        });

        Ok(pipeline)
    }
}
