use wgpu::{Device, ShaderSource, TextureFormat};

#[derive(Debug)]
pub struct ShadyDescriptor<'a> {
    pub device: &'a Device,
    pub initial_fragment_shader: Option<ShaderSource<'a>>,
    pub texture_format: TextureFormat,
    pub bind_group_index: u32,
    pub vertex_buffer_index: u32,
}
