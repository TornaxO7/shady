//! A [shadertoy] *like* library to be able to easily integrate [shadertoy]-*like* stuff in your applications.
//! It provides functions to setup the following uniform buffers (which will be also called `Resources` within this doc):
//!
//! - `iAudio`: Contains frequency bars of an audio source.
//! - `iFrame`: Contains the current frame count.
//! - `iMouse`: Contains the coordinate points of the user's mouse.
//! - `iResolution`: Contains the height and width of the surface which will be drawed on.
//! - `iTime`: The playback time of the shader.
//!
//! **Note:**
//! - You should be familiar with [wgpu] code in order to be able to use this.
//! - `shady` is not compatible with [shadertoy]'s shaders so you can't simply copy+paste the fragment code from [shadertoy] to
//!   application which are using `shady` (but porting them should be very easy in general).
//!
//! # Feature flags
//! Each resource is behind a feature gate so if you don't want to use some of them, just disable their feature gate.
//!
//! # Example
//! An "mini" example can be seen here: <https://github.com/TornaxO7/shady/blob/main/shady-lib/examples/mini-simple.rs>
//! I tried to make it as small as possible.
//!
//! Just search after the string `SHADY` within the file and you'll see what you can/need to do to include it into your renderer.
//!
//! [shadertoy]: https://www.shadertoy.com/
//! [wgpu]: https://crates.io/crates/wgpu
mod descriptor;
mod resources;
mod template;
mod vertices;

use resources::{Resource, Resources};
use tracing::instrument;
use wgpu::{CommandEncoder, Device, ShaderSource, TextureView};

pub use descriptor::ShadyDescriptor;

#[cfg(feature = "audio")]
pub use shady_audio;

#[cfg(feature = "mouse")]
pub use resources::MouseState;
pub use template::TemplateLang;

/// The name of the entrypoint function of the fragment shader for `shady`.
pub const FRAGMENT_ENTRYPOINT: &str = "main";

const BIND_GROUP_INDEX: u32 = 0;
const VBUFFER_INDEX: u32 = 0;

/// A wrapper around [wgpu::RenderPipeline].
#[derive(Debug, Clone)]
pub struct ShadyRenderPipeline(wgpu::RenderPipeline);

impl AsRef<ShadyRenderPipeline> for ShadyRenderPipeline {
    fn as_ref(&self) -> &Self {
        self
    }
}

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

    vbuffer: wgpu::Buffer,
    ibuffer: wgpu::Buffer,
}

// General functions
impl Shady {
    /// Create a new instance of `Shady`.
    #[instrument(level = "trace", skip_all)]
    pub fn new<'a>(desc: ShadyDescriptor) -> Self {
        let ShadyDescriptor { device, .. } = &desc;

        let resources = Resources::new(&desc);

        let bind_group = resources.bind_group(device);

        Self {
            resources,
            bind_group,
            vbuffer: vertices::vertex_buffer(device),
            ibuffer: vertices::index_buffer(device),
        }
    }

    /// Add a render pass to the given `encoder` and `texture_view`.
    pub fn add_render_pass(
        &self,
        encoder: &mut CommandEncoder,
        texture_view: &TextureView,
        pipelines: impl IntoIterator<Item = impl AsRef<ShadyRenderPipeline>>,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });

        render_pass.set_bind_group(BIND_GROUP_INDEX, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(VBUFFER_INDEX, self.vbuffer.slice(..));
        render_pass.set_index_buffer(self.ibuffer.slice(..), wgpu::IndexFormat::Uint16);

        for pipeline in pipelines.into_iter() {
            render_pass.set_pipeline(&pipeline.as_ref().0);
            render_pass.draw_indexed(vertices::index_buffer_range(), 0, 0..1);
        }
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
    pub fn set_audio_frequency_range(
        &mut self,
        sample_processor: &shady_audio::SampleProcessor,
        freq_range: std::ops::Range<std::num::NonZeroU16>,
    ) {
        self.resources
            .audio
            .set_frequency_range(sample_processor, freq_range);
    }

    /// Sets the amount of bar-values.
    ///
    /// # Affected uniform buffer
    /// `iAudio`
    #[inline]
    #[cfg(feature = "audio")]
    pub fn set_audio_bars(
        &mut self,
        device: &Device,
        sample_processor: &shady_audio::SampleProcessor,
        amount_bars: std::num::NonZeroUsize,
    ) {
        self.resources
            .audio
            .set_bars(device, sample_processor, amount_bars);
        // audio buffer will change => needs to be rebinded
        self.bind_group = self.resources.bind_group(device);
    }
}

/// Methods to overwrite/update the responding uniform buffer for the next time you render a frame with [Shady].
impl Shady {
    /// Updates the `iAudio` uniform buffer with new values.
    #[inline]
    #[cfg(feature = "audio")]
    pub fn update_audio_buffer(
        &mut self,
        queue: &wgpu::Queue,
        sample_processor: &shady_audio::SampleProcessor,
    ) {
        self.resources.audio.fetch_audio(sample_processor);
        self.resources.audio.update_buffer(queue);
    }

    /// Updates the `iFrame` uniform buffer with new values.
    #[inline]
    #[cfg(feature = "frame")]
    pub fn update_frame_buffer(&mut self, queue: &wgpu::Queue) {
        self.resources.frame.update_buffer(queue);
    }

    /// Updates the `iMouse` uniform buffer with new values.
    #[inline]
    #[cfg(feature = "mouse")]
    pub fn update_mouse_buffer(&mut self, queue: &wgpu::Queue) {
        self.resources.mouse.update_buffer(queue);
    }

    /// Updates the `iResolution` uniform buffer with new values.
    #[inline]
    #[cfg(feature = "resolution")]
    pub fn update_resolution_buffer(&mut self, queue: &wgpu::Queue) {
        self.resources.resolution.update_buffer(queue);
    }

    /// Updates the `iTime` uniform buffer with new values.
    #[inline]
    #[cfg(feature = "time")]
    pub fn update_time_buffer(&mut self, queue: &wgpu::Queue) {
        self.resources.time.update_buffer(queue);
    }
}

/// Creates a pre-configured pipeline which can then be used in [Shady::add_render_pass].
pub fn create_render_pipeline<'a>(
    device: &Device,
    shader_source: ShaderSource<'a>,
    texture_format: &'a wgpu::TextureFormat,
) -> ShadyRenderPipeline {
    let bind_group_layout = Resources::bind_group_layout(device);
    let pipeline = get_render_pipeline(device, shader_source, bind_group_layout, texture_format);

    ShadyRenderPipeline(pipeline)
}

fn get_render_pipeline(
    device: &Device,
    shader_source: ShaderSource<'_>,
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

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Shady render pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &vertex_shader,
            entry_point: Some("vertex_main"),
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
            entry_point: Some("main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: *texture_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        multiview: None,
        cache: None,
    })
}
