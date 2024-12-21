mod frontend;
mod uniforms;
mod vertices;

use std::borrow::Cow;
use tracing::instrument;
use uniforms::Uniforms;
use wgpu::Device;

pub use frontend::{Frontend, GlslFrontend, WgslFrontend};
pub use vertices::{index_buffer, index_buffer_range, vertex_buffer, BUFFER_LAYOUT};

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

pub struct Shady<F: Frontend> {
    uniforms: Uniforms,
    pub bind_group: wgpu::BindGroup,
    frontend: F,
}

// General functions
impl<F: Frontend> Shady<F> {
    #[instrument(level = "trace")]
    pub fn new(device: &Device) -> Self {
        let uniforms = Uniforms::new(device);

        let bind_group = uniforms.bind_group(device);

        Self {
            uniforms,
            bind_group,
            frontend: F::new(),
        }
    }

    #[instrument(skip_all, level = "trace")]
    pub fn cleanup(&mut self) {
        self.uniforms.cleanup();
    }

    #[instrument(skip(self, device, fragment_shader), level = "trace")]
    pub fn get_render_pipeline<S: AsRef<str>>(
        &mut self,
        device: &Device,
        fragment_shader: S,
        texture_format: &wgpu::TextureFormat,
    ) -> Result<wgpu::RenderPipeline, Error> {
        let vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shady vertex shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let fragment_shader = {
            let fragment_module = self.frontend.parse(fragment_shader.as_ref())?;

            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shady fragment shader"),
                source: wgpu::ShaderSource::Naga(Cow::Owned(fragment_module)),
            })
        };

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Shady pipeline layout"),
            bind_group_layouts: &[&Uniforms::bind_group_layout(device)],
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

/// Updating functions
impl<F: Frontend> Shady<F> {
    pub fn update_resolution(&mut self, width: u32, height: u32) {
        self.uniforms.resolution.update_resolution(width, height);
    }

    pub fn update_buffers(&mut self, queue: &mut wgpu::Queue) {
        self.uniforms.update_buffers(queue);
    }
}
