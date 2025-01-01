mod descriptor;
mod resources;
mod shader_language;
mod vertices;

use resources::Resources;
use std::borrow::Cow;
use tracing::instrument;
use wgpu::{CommandEncoder, Device, TextureView};

pub use descriptor::ShadyDescriptor;
pub use resources::MouseState;
pub use shader_language::{Glsl, ShaderLanguage, Wgsl};
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

pub struct Shady<S: ShaderLanguage> {
    resources: Resources,
    pub bind_group: wgpu::BindGroup,
    shader_parser: S,

    pipeline: wgpu::RenderPipeline,
    bind_group_index: u32,
    vbuffer_index: u32,

    texture_format: wgpu::TextureFormat,

    vbuffer: wgpu::Buffer,
    ibuffer: wgpu::Buffer,
}

// General functions
impl<SL: ShaderLanguage> Shady<SL> {
    #[instrument(level = "trace")]
    pub fn new(desc: &ShadyDescriptor) -> Result<Self, Error> {
        let ShadyDescriptor {
            device,
            fragment_shader,
            texture_format,
            bind_group_index,
            vertex_buffer_index,
        } = desc;

        let resources = Resources::new(device);
        let mut shader_parser = SL::new();

        let bind_group = resources.bind_group(device);

        let pipeline = {
            let bind_group_layout = resources.bind_group_layout(device);

            get_render_pipeline(
                device,
                fragment_shader,
                &mut shader_parser,
                bind_group_layout,
                texture_format,
            )
        }?;

        Ok(Self {
            resources,
            bind_group,
            shader_parser,
            pipeline,
            texture_format: *texture_format,
            bind_group_index: *bind_group_index,
            vbuffer_index: *vertex_buffer_index,
            vbuffer: vertices::vertex_buffer(device),
            ibuffer: vertices::index_buffer(device),
        })
    }

    pub fn add_render_pass(&self, encoder: &mut CommandEncoder, texture_view: &TextureView) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(self.bind_group_index, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(self.vbuffer_index, self.vbuffer.slice(..));
        render_pass.set_index_buffer(self.ibuffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(crate::index_buffer_range(), 0, 0..1);
    }

    #[instrument(skip(self, device, fragment_shader), level = "trace")]
    pub fn update_render_pipeline<S: AsRef<str>>(
        &mut self,
        device: &Device,
        fragment_shader: S,
    ) -> Result<(), Error> {
        self.resources.frame.reset_counter();
        let bind_group_layout = self.resources.bind_group_layout(device);

        self.pipeline = get_render_pipeline(
            device,
            fragment_shader,
            &mut self.shader_parser,
            bind_group_layout,
            &self.texture_format,
        )?;

        Ok(())
    }
}

/// Updating functions
impl<F: ShaderLanguage> Shady<F> {
    pub fn update_resolution(&mut self, width: u32, height: u32) {
        debug_assert!(width > 0);
        debug_assert!(height > 0);
        self.resources.resolution.update_resolution(width, height);
    }

    pub fn update_mouse_input(&mut self, state: MouseState) {
        self.resources.mouse.mouse_input(state);
    }

    pub fn update_cursor(&mut self, x: f32, y: f32) {
        self.resources.mouse.cursor_moved(x, y);
    }

    pub fn prepare_next_frame(&mut self, queue: &mut wgpu::Queue) {
        self.resources.frame.next_frame();
        self.resources.update_buffers(queue);
        self.resources.audio.fetch_audio();
    }
}

pub fn get_render_pipeline<S: AsRef<str>, SL: ShaderLanguage>(
    device: &Device,
    fragment_shader: S,
    shader_parser: &mut SL,
    bind_group_layout: wgpu::BindGroupLayout,
    texture_format: &wgpu::TextureFormat,
) -> Result<wgpu::RenderPipeline, Error> {
    let vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shady vertex shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("vertex_shader.wgsl").into()),
    });

    let fragment_shader = {
        let fragment_module = shader_parser.parse(fragment_shader.as_ref())?;

        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shady fragment shader"),
            source: wgpu::ShaderSource::Naga(Cow::Owned(fragment_module)),
        })
    };

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Shady pipeline layout"),
        bind_group_layouts: &[&bind_group_layout],
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
