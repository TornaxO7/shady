use wgpu::{Device, TextureFormat};

#[derive(Debug)]
pub struct ShadyDescriptor<'a> {
    pub device: &'a Device,
    pub fragment_shader: &'a str,
    pub texture_format: TextureFormat,
    pub bind_group_index: u32,
    pub vertex_buffer_index: u32,
}
