mod descriptor;
mod resources;
mod template;
mod vertices;

use resources::{Resource, Resources};
use std::{
    num::{NonZeroU32, NonZeroUsize},
    ops::Range,
};
use tracing::{debug, instrument};
use wgpu::{CommandEncoder, Device, ShaderSource, TextureView};

pub use descriptor::ShadyDescriptor;

#[cfg(feature = "mouse")]
pub use resources::MouseState;
pub use template::TemplateLang;
pub use vertices::{index_buffer, index_buffer_range, vertex_buffer, BUFFER_LAYOUT};

pub const FRAGMENT_ENTRYPOINT: &str = "main";

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
    pub fn new<'a>(desc: ShadyDescriptor) -> Self {
        let ShadyDescriptor {
            device,
            initial_fragment_shader,
            texture_format,
            bind_group_index,
            vertex_buffer_index,
        } = desc;

        let resources = Resources::new(device);

        let bind_group = resources.bind_group(device);

        let pipeline = initial_fragment_shader.map(|shader| {
            let bind_group_layout = Resources::bind_group_layout(device);

            get_render_pipeline(device, shader, bind_group_layout, &texture_format)
        });

        Self {
            resources,
            bind_group,
            pipeline,
            texture_format,
            bind_group_index,
            vbuffer_index: vertex_buffer_index,
            vbuffer: vertices::vertex_buffer(device),
            ibuffer: vertices::index_buffer(device),
        }
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

            debug!("Applied renderpass");
        } else {
            debug!("Pipeline not set!");
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

/// Update buffer functions
impl Shady {
    #[inline]
    #[cfg(feature = "audio")]
    pub fn update_audio_buffer(&mut self, queue: &mut wgpu::Queue) {
        self.resources.audio.fetch_audio();
        self.resources.audio.update_buffer(queue);
    }

    #[inline]
    #[cfg(feature = "frame")]
    pub fn update_frame_buffer(&mut self, queue: &mut wgpu::Queue) {
        self.resources.frame.update_buffer(queue);
    }

    #[inline]
    #[cfg(feature = "mouse")]
    pub fn update_mouse_buffer(&mut self, queue: &mut wgpu::Queue) {
        self.resources.mouse.update_buffer(queue);
    }

    #[inline]
    #[cfg(feature = "resolution")]
    pub fn update_resolution_buffer(&mut self, queue: &mut wgpu::Queue) {
        self.resources.resolution.update_buffer(queue);
    }

    #[inline]
    #[cfg(feature = "time")]
    pub fn update_time_buffer(&mut self, queue: &mut wgpu::Queue) {
        self.resources.time.update_buffer(queue);
    }
}

/// Setter functions
impl Shady {
    #[inline]
    #[cfg(feature = "resolution")]
    pub fn set_resolution(&mut self, width: u32, height: u32) {
        debug_assert!(width > 0);
        debug_assert!(height > 0);
        self.resources.resolution.set(width, height);
    }

    #[inline]
    #[cfg(feature = "mouse")]
    pub fn set_mouse_state(&mut self, state: MouseState) {
        self.resources.mouse.set_state(state);
    }

    #[inline]
    #[cfg(feature = "mouse")]
    pub fn set_mouse_pos(&mut self, x: f32, y: f32) {
        self.resources.mouse.set_pos(x, y);
    }

    #[inline]
    #[cfg(feature = "frame")]
    pub fn inc_frame(&mut self) {
        self.resources.frame.inc();
    }

    #[inline]
    #[cfg(feature = "audio")]
    pub fn set_audio_frequency_range(&mut self, freq_range: Range<NonZeroU32>) -> Result<(), ()> {
        self.resources.audio.set_frequency_range(freq_range)
    }

    #[inline]
    #[cfg(feature = "audio")]
    pub fn set_audio_bars(&mut self, amount_bars: NonZeroUsize) {
        self.resources.audio.set_bars(amount_bars);
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
