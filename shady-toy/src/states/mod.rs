use shady::ShaderLanguage;

pub mod inner;
pub mod texture_state;
pub mod window_state;

pub trait RenderState<S: ShaderLanguage> {
    fn prepare_next_frame(&mut self);

    fn render(&mut self) -> Result<(), wgpu::SurfaceError>;

    fn update_pipeline(&mut self, fragment_code: &str) -> Result<(), shady::Error>;

    fn shady_mut(&mut self) -> &mut shady::Shady<S>;
}
