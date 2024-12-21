mod frontend;
mod uniforms;
mod vertices;

use std::{borrow::Cow, ops::Range};

use frontend::{Frontend, GlslFrontend, WgslFrontend};
use uniforms::Uniforms;
use wgpu::{naga::Module, Device};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid fragment shader in line {line_num}, column {line_pos}: {msg}")]
    InvalidWgslFragmentShader {
        fragment_code: String,
        msg: String,
        line_num: u32,
        line_pos: u32,
        offset: u32,
        length: u32,
    },

    #[error("Invalid fragment shader: {0}")]
    InvalidGlslFragmentShader(String),
}

pub struct Shady {
    pub vbuffer: wgpu::Buffer,
    pub ibuffer: wgpu::Buffer,
    uniforms: Uniforms,

    wgsl_frontend: WgslFrontend,
    glsl_frontend: GlslFrontend,
}

impl Shady {
    pub fn new(device: &Device) -> Self {
        Self {
            vbuffer: vertices::get_vertex_buffer(device),
            ibuffer: vertices::get_index_buffer(device),
            uniforms: Uniforms::new(device),

            wgsl_frontend: WgslFrontend::new(),
            glsl_frontend: GlslFrontend::new(),
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

    pub fn get_wgsl_pipeline<S: AsRef<str>>(
        &mut self,
        device: &Device,
        fragment_shader: S,
        texture_format: &wgpu::TextureFormat,
    ) -> Result<wgpu::RenderPipeline, Error> {
        let source = fragment_shader.as_ref();
        let fragment_module = self.wgsl_frontend.parse(source)?;

        Ok(self.get_render_pipeline(device, fragment_module, texture_format))
    }

    pub fn get_glsl_pipeline<S: AsRef<str>>(
        &mut self,
        device: &Device,
        fragment_shader: S,
        texture_format: &wgpu::TextureFormat,
    ) -> Result<wgpu::RenderPipeline, Error> {
        let source = fragment_shader.as_ref();
        let fragment_module = self.glsl_frontend.parse(source)?;

        Ok(self.get_render_pipeline(device, fragment_module, texture_format))
    }

    fn get_render_pipeline(
        &mut self,
        device: &Device,
        fragment_module: Module,
        texture_format: &wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shady vertex shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let fragment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shady fragment shader"),
            source: wgpu::ShaderSource::Naga(Cow::Owned(fragment_module)),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Shady pipeline layout"),
            bind_group_layouts: &[&self.uniforms.bind.layout],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
        })
    }
}
