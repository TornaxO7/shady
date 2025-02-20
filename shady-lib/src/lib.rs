mod descriptor;
mod resources;
mod shader_language;
mod template;
mod vertices;

use resources::Resources;
use std::fmt;
use template::TemplateGenerator;
use tracing::instrument;
use wgpu::{CommandEncoder, Device, ShaderSource, TextureView};

pub use descriptor::ShadyDescriptor;

#[cfg(feature = "mouse")]
pub use resources::MouseState;
pub use shader_language::{Glsl, ShaderParser, Wgsl};
pub use template::TemplateLang;
pub use vertices::{index_buffer, index_buffer_range, vertex_buffer, BUFFER_LAYOUT};

pub const FRAGMENT_ENTRYPOINT: &str = "main";

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
    resources: Resources,
    bind_group: wgpu::BindGroup,

    pipeline: Option<wgpu::RenderPipeline>,
    bind_group_index: u32,
    vbuffer_index: u32,

    texture_format: wgpu::TextureFormat,

    vbuffer: wgpu::Buffer,
    ibuffer: wgpu::Buffer,
}

// General functions
impl Shady {
    #[instrument(level = "trace")]
    pub fn new<'a>(desc: ShadyDescriptor) -> Result<Self, Error> {
        let ShadyDescriptor {
            device,
            shader_source,
            texture_format,
            bind_group_index,
            vertex_buffer_index,
        } = desc;

        let resources = Resources::new(device);

        let bind_group = resources.bind_group(device);

        let pipeline = shader_source.map(|shader| {
            let bind_group_layout = Resources::bind_group_layout(device);

            get_render_pipeline(device, shader, bind_group_layout, &texture_format)
        });

        Ok(Self {
            resources,
            bind_group,
            pipeline,
            texture_format,
            bind_group_index,
            vbuffer_index: vertex_buffer_index,
            vbuffer: vertices::vertex_buffer(device),
            ibuffer: vertices::index_buffer(device),
        })
    }

    pub fn add_render_pass(&self, encoder: &mut CommandEncoder, texture_view: &TextureView) {
        if let Some(pipeline) = &self.pipeline {
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

            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(self.bind_group_index, &self.bind_group, &[]);
            render_pass.set_vertex_buffer(self.vbuffer_index, self.vbuffer.slice(..));
            render_pass.set_index_buffer(self.ibuffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(crate::index_buffer_range(), 0, 0..1);
        }
    }

    #[instrument(skip(self, device), level = "trace")]
    pub fn update_render_pipeline<'a>(&mut self, device: &Device, shader_source: ShaderSource<'a>) {
        #[cfg(feature = "frame")]
        self.resources.frame.reset_counter();
        let bind_group_layout = Resources::bind_group_layout(device);

        self.pipeline = Some(get_render_pipeline(
            device,
            shader_source,
            bind_group_layout,
            &self.texture_format,
        ));
    }
}

/// Updating functions
impl Shady {
    #[cfg(feature = "resolution")]
    pub fn update_resolution(&mut self, width: u32, height: u32) {
        debug_assert!(width > 0);
        debug_assert!(height > 0);
        self.resources.resolution.update_resolution(width, height);
    }

    #[cfg(feature = "mouse")]
    pub fn update_mouse_input(&mut self, state: MouseState) {
        self.resources.mouse.mouse_input(state);
    }

    #[cfg(feature = "mouse")]
    pub fn update_cursor(&mut self, x: f32, y: f32) {
        self.resources.mouse.cursor_moved(x, y);
    }

    #[cfg(feature = "frame")]
    pub fn prepare_next_frame(&mut self, queue: &mut wgpu::Queue) {
        self.resources.frame.next_frame();
        self.resources.update_buffers(queue);
        self.resources.audio.fetch_audio();
    }
}

fn get_render_pipeline<'a>(
    device: &Device,
    shader_source: ShaderSource<'a>,
    bind_group_layout: wgpu::BindGroupLayout,
    texture_format: &wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shady vertex shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("vertex_shader.wgsl").into()),
    });

    let fragment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shady fragment shader"),
        source: shader_source,
    });

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

    pipeline
}

pub fn get_template(
    lang: TemplateLang,
    writer: &mut dyn std::fmt::Write,
) -> Result<(), fmt::Error> {
    match lang {
        TemplateLang::Wgsl { bind_group_index } => {
            Resources::write_wgsl_template(writer, bind_group_index)?;

            writer.write_fmt(format_args!(
                "
@fragment
fn {}(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {{
    let uv = pos.xy/iResolution.xy;
    let col = 0.5 + 0.5 * cos(iTime + uv.xyx + vec3<f32>(0.0, 2.0, 4.0));

    return vec4<f32>(col, 1.0);
}}
",
                FRAGMENT_ENTRYPOINT
            ))?;
        }

        TemplateLang::Glsl => {
            Resources::write_glsl_template(writer)?;

            writer.write_fmt(format_args!(
                "
// the color which the pixel should have
layout(location = 0) out vec4 fragColor;

void {}() {{
    // Normalized pixel coordinates (from 0 to 1)
    vec2 uv = gl_FragCoord.xy/iResolution.xy;

    // Time varying pixel color
    vec3 col = 0.5 + 0.5*cos(iTime+uv.xyx+vec3(0,2,4));

    // Output to screen
    fragColor = vec4(col,1.0);      
}}
",
                FRAGMENT_ENTRYPOINT
            ))?;
        }
    };

    Ok(())
}
