mod glsl;
mod wgsl;

pub use glsl::Glsl;
pub use wgsl::Wgsl;

pub trait ShaderLanguage {
    fn new() -> Self;

    fn parse(&mut self, fragment_shader: &str) -> Result<wgpu::naga::Module, crate::Error>;
}