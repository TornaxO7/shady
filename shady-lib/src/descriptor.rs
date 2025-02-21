use wgpu::{Device, ShaderSource, TextureFormat};

/// Describes [Shady] for [Shady::new]
#[derive(Debug)]
pub struct ShadyDescriptor<'a> {
    /// The [wgpu::Device] which `shady` is going to render with.
    pub device: &'a Device,

    /// You can provide an initial fragment shader which will let [Shady] create
    /// its pipeline from the beginning. However, you can change the fragment shader
    /// afterwards.
    pub initial_fragment_shader: Option<ShaderSource<'a>>,

    /// The texture format of the texture where [Shady]'s pipeline will render to.
    pub texture_format: TextureFormat,

    /// The "bind group"/"layout" value which the uniform buffer of the fragment shader will assigned to.
    pub bind_group_index: u32,

    /// The index of the vertex buffer where [Shady] will add its vertices.
    pub vertex_buffer_index: u32,
}
