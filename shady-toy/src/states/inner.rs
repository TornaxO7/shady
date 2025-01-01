use shady::{ShaderLanguage, Shady};
use wgpu::{CommandEncoder, Device, Queue, RenderPipeline, TextureFormat, TextureView};

pub struct InnerState<S: ShaderLanguage> {
    pub shady: Shady<S>,
    pipeline: RenderPipeline,

    vbuffer: wgpu::Buffer,
    ibuffer: wgpu::Buffer,
    texture_format: TextureFormat,
}

impl<S: ShaderLanguage> InnerState<S> {
    pub fn new(
        device: &Device,
        fragment_code: &str,
        texture_format: TextureFormat,
    ) -> Result<Self, shady::Error> {
        let mut shady = Shady::new(device);
        let pipeline = shady.get_render_pipeline(device, fragment_code, &texture_format)?;

        let vbuffer = shady::vertex_buffer(device);
        let ibuffer = shady::index_buffer(device);

        Ok(Self {
            shady,
            pipeline,
            vbuffer,
            ibuffer,
            texture_format,
        })
    }

    pub fn update_pipeline(
        &mut self,
        device: &Device,
        fragment_code: &str,
    ) -> Result<(), shady::Error> {
        self.pipeline =
            self.shady
                .get_render_pipeline(device, fragment_code, &self.texture_format)?;

        Ok(())
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        if new_width > 0 && new_height > 0 {
            self.shady.update_resolution(new_width, new_height);
        }
    }

    pub fn prepare_next_frame(&mut self, queue: &mut Queue) {
        self.shady.prepare_next_frame(queue)
    }

    pub fn apply_renderpass(&self, encoder: &mut CommandEncoder, texture_view: &TextureView) {
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.shady.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vbuffer.slice(..));
            render_pass.set_index_buffer(self.ibuffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(shady::index_buffer_range(), 0, 0..1);
        }
    }
}
