//! A [shadertoy] *like* library to be able to easily integrate [shadertoy]-*like* stuff in your applications.
//! It provides functions to setup the following uniform buffers (which will be also called `Resources` within this doc):
//!
//! - `iAudio`: Contains frequency bars of an audio source.
//! - `iFrame`: Contains the current frame count.
//! - `iMouse`: Contains the coordinate points of the user's mouse.
//! - `iResolution`: Contains the height and width of the surface which will be drawed on.
//! - `iTime`: The playback time of the shader.
//!
//! **Note:** You should be familiar with [wgpu] code in order to be able to use this.
//!
//! # Feature flags
//! Each resource is behind a feature gate so if you don't want to use some of them, just disable their feature gate.
//!
//! # Example
//! An (mini) example can be seen here: <https://github.com/TornaxO7/shady/blob/main/shady-lib/examples/mini-simple.rs>
//!
//! But here's a rough structure how it's meant to be used:
//!
//! ```ignore
//! use shady::{Shady, ShadyDescriptor};
//!
//! struct State {
//!     shady: Shady,
//!
//!     // ... and your other wgpu stuff, like `wgpu::Device`, etc.
//!     queue: wgpu::Queue,
//!     device: wgpu::Device,
//! }
//!
//! impl State {
//!     pub fn new() -> Self {
//!         // .. your stuff
//!
//!         let shady = Shady::new(ShadyDescriptor {
//!             // ... set the attributes
//!         });
//!
//!         // ...
//!     }
//!
//!     pub fn prepare_next_frame(&mut self) {
//!         // here you can change some properties of shady or change
//!         // the values of the uniform buffers of the fragment shader
//!         self.shady.inc_frame();
//!
//!         // ... afterwards tell shady to move the values into the uniform buffer
//!         self.shady.update_frame_buffer(&mut self.queue);
//!         // ... and other buffers you'd like to update
//!     }
//!
//!     pub fn render(&mut self) {
//!         // ...
//!
//!         let view = ...;
//!         let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
//!            label: Some("Some random encoder"),
//!         });
//!
//!         // shady will add a render pass and you are good to go!
//!         self.shady.add_render_pass(&mut encoder, &view);
//!     }
//!
//!     pub fn load_fragment_code<'a>(&mut self, shader_source: wgpu::ShaderSource<'a>) {
//!         // set the render pipeline which should execute the given fragment code
//!         self.shady.set_render_pipeline(&self.device, shader_source);
//!     }
//! }
//! ```
//!
//! [shadertoy]: https://www.shadertoy.com/
//! [wgpu]: https://crates.io/crates/wgpu
mod descriptor;
mod resources;
mod template;
mod vertices;

use resources::{Resource, Resources};
use shady_audio::fetcher::Fetcher;
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

/// The name of the entrypoint function of the fragment shader for `shady`.
pub const FRAGMENT_ENTRYPOINT: &str = "main";

/// The main struct of this crate.
///
/// # Example
/// It's recommended to take a look into the [mini-simple.rs] example to understand its usage and methods.
/// Open the search of your web-browser and enter `SHADY`. Those are the places where you could use the struct.
///
/// [mini-simple.rs]: https://github.com/TornaxO7/shady/blob/main/shady-lib/examples/mini-simple.rs
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
    /// Create a new instance of `Shady`.
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

    /// Add a render pass to the given `encoder` and `texture_view`.
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
            render_pass.draw_indexed(vertices::index_buffer_range(), 0, 0..1);

            debug!("Applied renderpass");
        } else {
            debug!("Pipeline not set!");
        }
    }

    /// Sets/Updates the render pipeline of [Shady].
    /// Is especially used if you'd like to display a different fragment shader with [Shady].
    /// To get a fragment shader which [Shady] will be able to use see [TemplateLang].
    #[instrument(skip(self, device), level = "trace")]
    pub fn set_render_pipeline<'a>(&mut self, device: &Device, shader_source: ShaderSource<'a>) {
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

/// Methods to set/change some values in [Shady]'s internal stage which will be then written
/// into the uniform buffer after calling their responsible `update_*` function.
///
/// # Example
/// ```ignore
/// let mut shady = Shady::new(...);
///
/// // let's assume the surface has changed.
/// // Tell that shady:
/// shady.set_resolution(100, 200);
///
/// // now write that new resolution into the `iResolution` uniform buffer
/// shady.update_resolution_buffer(...);
/// ```
impl Shady {
    /// Set the resolution of the output screen.
    ///
    /// # Affected uniform buffer
    /// `iResolution`
    #[inline]
    #[cfg(feature = "resolution")]
    pub fn set_resolution(&mut self, width: u32, height: u32) {
        debug_assert!(width > 0);
        debug_assert!(height > 0);
        self.resources.resolution.set(width, height);
    }

    /// Set the mouse state.
    ///
    /// # Affected uniform buffer
    /// `iMouse`
    #[inline]
    #[cfg(feature = "mouse")]
    pub fn set_mouse_state(&mut self, state: MouseState) {
        self.resources.mouse.set_state(state);
    }

    /// Set the mouse position.
    ///
    /// # Affected uniform buffer
    /// `iMouse`
    #[inline]
    #[cfg(feature = "mouse")]
    pub fn set_mouse_pos(&mut self, x: f32, y: f32) {
        self.resources.mouse.set_pos(x, y);
    }

    /// Increment the frame counter.
    ///
    /// # Affected uniform buffer
    /// `iFrame`
    #[inline]
    #[cfg(feature = "frame")]
    pub fn inc_frame(&mut self) {
        self.resources.frame.inc();
    }

    /// Set the frequency range which [Shady] should listen to from the sample fetcher.
    ///
    /// # Affected uniform buffer
    /// `iAudio`
    #[inline]
    #[cfg(feature = "audio")]
    pub fn set_audio_frequency_range(&mut self, freq_range: Range<NonZeroU32>) -> Result<(), ()> {
        self.resources.audio.set_frequency_range(freq_range)
    }

    /// Sets the amount of bar-values.
    ///
    /// # Affected uniform buffer
    /// `iAudio`
    #[inline]
    #[cfg(feature = "audio")]
    pub fn set_audio_bars(&mut self, amount_bars: NonZeroUsize) {
        self.resources.audio.set_bars(amount_bars);
    }

    /// Set the audio fetcher which [Shady] should use.
    ///
    /// # Affected uniform buffer
    /// `iAudio`
    pub fn set_audio_fetcher(&mut self, fetcher: Box<dyn Fetcher>) {
        self.resources.audio.set_fetcher(fetcher);
    }
}

/// Methods to overwrite/update the responding uniform buffer for the next time you render a frame with [Shady].
impl Shady {
    /// Updates the `iAudio` uniform buffer with new values.
    #[inline]
    #[cfg(feature = "audio")]
    pub fn update_audio_buffer(&mut self, queue: &mut wgpu::Queue) {
        self.resources.audio.fetch_audio();
        self.resources.audio.update_buffer(queue);
    }

    /// Updates the `iFrame` uniform buffer with new values.
    #[inline]
    #[cfg(feature = "frame")]
    pub fn update_frame_buffer(&mut self, queue: &mut wgpu::Queue) {
        self.resources.frame.update_buffer(queue);
    }

    /// Updates the `iMouse` uniform buffer with new values.
    #[inline]
    #[cfg(feature = "mouse")]
    pub fn update_mouse_buffer(&mut self, queue: &mut wgpu::Queue) {
        self.resources.mouse.update_buffer(queue);
    }

    /// Updates the `iResolution` uniform buffer with new values.
    #[inline]
    #[cfg(feature = "resolution")]
    pub fn update_resolution_buffer(&mut self, queue: &mut wgpu::Queue) {
        self.resources.resolution.update_buffer(queue);
    }

    /// Updates the `iTime` uniform buffer with new values.
    #[inline]
    #[cfg(feature = "time")]
    pub fn update_time_buffer(&mut self, queue: &mut wgpu::Queue) {
        self.resources.time.update_buffer(queue);
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
