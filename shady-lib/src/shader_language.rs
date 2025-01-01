use tracing::instrument;
use wgpu::naga::{front::glsl::Options, ShaderStage};

pub trait ShaderLanguage {
    fn new() -> Self;

    fn parse(&mut self, fragment_shader: &str) -> Result<wgpu::naga::Module, crate::Error>;
}

pub struct Wgsl(wgpu::naga::front::wgsl::Frontend);

impl Wgsl {
    #[instrument(level = "trace")]
    pub fn new() -> Self {
        Self(wgpu::naga::front::wgsl::Frontend::new())
    }
}

impl ShaderLanguage for Wgsl {
    #[instrument(level = "trace")]
    fn new() -> Self {
        Self::new()
    }

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

pub struct Glsl(wgpu::naga::front::glsl::Frontend);

impl Glsl {
    #[instrument(level = "trace")]
    pub fn new() -> Self {
        Self(wgpu::naga::front::glsl::Frontend::default())
    }
}

impl ShaderLanguage for Glsl {
    #[instrument(level = "trace")]
    fn new() -> Self {
        Self::new()
    }

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
