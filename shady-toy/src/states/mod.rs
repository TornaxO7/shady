use wgpu::ShaderSource;

#[cfg(test)]
mod texture_state;
pub mod window_state;

const SHADY_BIND_GROUP_INDEX: u32 = 0;
const SHADY_VERTEX_BUFFER_INDEX: u32 = 0;

pub trait RenderState<'a> {
    fn prepare_next_frame(&mut self);

    fn render(&mut self) -> Result<(), wgpu::SurfaceError>;

    fn update_pipeline(&mut self, shader_source: ShaderSource<'a>);
}
