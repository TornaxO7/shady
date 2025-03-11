use shady_audio::SampleProcessor;
use wgpu::Device;

/// Describes [Shady] for [Shady::new]
///
/// [Shady]: crate::Shady
/// [Shady::new]: crate::Shady::new
pub struct ShadyDescriptor<'a> {
    /// The [wgpu::Device] which `shady` is going to render with.
    pub device: &'a Device,

    #[cfg(feature = "audio")]
    pub sample_processor: &'a SampleProcessor,
}
