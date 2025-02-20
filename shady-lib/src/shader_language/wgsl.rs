use tracing::instrument;

use super::ShaderParser;

pub struct Wgsl(wgpu::naga::front::wgsl::Frontend);

impl Wgsl {
    #[instrument(level = "trace")]
    pub fn new() -> Self {
        Self(wgpu::naga::front::wgsl::Frontend::new())
    }

    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
}

impl ShaderParser for Wgsl {
    #[instrument(skip(self), level = "trace")]
    fn parse(&mut self, fragment_shader: &str) -> Result<wgpu::naga::Module, crate::Error> {
        self.0.parse(fragment_shader).map_err(|err| {
            let msg = err.message().to_string();
            let location = err.location(fragment_shader).unwrap();

            crate::Error::InvalidWgslFragmentShader {
                msg,
                fragment_code: fragment_shader.to_string(),
                line_num: location.line_number,
                line_pos: location.line_position,
                offset: location.offset,
                length: location.length,
            }
        })
    }
}
