use tracing::instrument;
use wgpu::naga::{front::glsl::Options, ShaderStage};

use super::ShaderParser;

pub struct Glsl(wgpu::naga::front::glsl::Frontend);

impl Glsl {
    #[instrument(level = "trace")]
    pub fn new() -> Self {
        Self(wgpu::naga::front::glsl::Frontend::default())
    }

    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
}

impl ShaderParser for Glsl {
    #[instrument(skip(self), level = "trace")]
    fn parse(&mut self, fragment_shader: &str) -> Result<wgpu::naga::Module, crate::Error> {
        let parse_options = Options::from(ShaderStage::Fragment);

        self.0
            .parse(&parse_options, fragment_shader)
            .map_err(|err| {
                crate::Error::InvalidGlslFragmentShader(err.emit_to_string(fragment_shader))
            })
    }
}
