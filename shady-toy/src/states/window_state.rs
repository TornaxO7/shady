use std::sync::Arc;

use pollster::FutureExt;
use shady::{
    shady_audio::{fetcher::SystemAudioFetcher, SampleProcessor},
    Shady, ShadyDescriptor,
};
use tracing::instrument;
use wgpu::{
    Backends, Device, Instance, Queue, ShaderSource, Surface, SurfaceConfiguration,
    TextureViewDescriptor,
};
use winit::{dpi::PhysicalSize, window::Window};

use super::RenderState;

pub struct WindowState<'a> {
    surface: Surface<'a>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    pipeline: Option<shady::ShadyRenderPipeline>,
    window: Arc<Window>,
    pub shady: Shady,
    sample_processor: SampleProcessor,
}

impl<'a> WindowState<'a> {
    pub fn new(window: Window, shader_source: Option<ShaderSource>) -> Self {
        let window = Arc::new(window);

        let instance = Instance::new(&wgpu::InstanceDescriptor {
            backends: Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance
            .create_surface(window.clone())
            .expect("Create surface from window.");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .block_on()
            .expect("Create wgpu-adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .block_on()
            .expect("Retrieve device and queue");

        let (config, shady, pipeline, sample_processor) = {
            let surface_caps = surface.get_capabilities(&adapter);
            let surface_format = surface_caps
                .formats
                .iter()
                .find(|f| f.is_srgb())
                .copied()
                .unwrap_or(surface_caps.formats[0]);

            let size = window.clone().inner_size();

            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width: size.width,
                height: size.height,
                present_mode: wgpu::PresentMode::AutoVsync,
                alpha_mode: surface_caps.alpha_modes[0],
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };

            let pipeline = shader_source
                .map(|source| shady::create_render_pipeline(&device, source, &surface_format));

            let sample_processor =
                SampleProcessor::new(SystemAudioFetcher::default(|err| panic!("{}", err)).unwrap());
            let shady = Shady::new(ShadyDescriptor {
                device: &device,
                sample_processor: &sample_processor,
            });

            (config, shady, pipeline, sample_processor)
        };

        surface.configure(&device, &config);

        Self {
            surface,
            device,
            queue,
            config,
            window,
            sample_processor,
            shady,
            pipeline,
        }
    }

    pub fn window(&self) -> Arc<Window> {
        self.window.clone()
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            #[cfg(feature = "resolution")]
            self.shady.set_resolution(new_size.width, new_size.height);
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
}

impl<'a> RenderState<'a> for WindowState<'a> {
    fn prepare_next_frame(&mut self) {
        #[cfg(feature = "frame")]
        self.shady.inc_frame();

        #[cfg(feature = "audio")]
        {
            self.sample_processor.process_next_samples();
            self.shady
                .update_audio_buffer(&self.queue, &self.sample_processor);
        }
        #[cfg(feature = "frame")]
        self.shady.update_frame_buffer(&self.queue);
        #[cfg(feature = "mouse")]
        self.shady.update_mouse_buffer(&self.queue);
        #[cfg(feature = "resolution")]
        self.shady.update_resolution_buffer(&self.queue);
        #[cfg(feature = "time")]
        self.shady.update_time_buffer(&self.queue);
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        if let Some(pipeline) = &self.pipeline {
            let output = self.surface.get_current_texture()?;
            let view = output
                .texture
                .create_view(&TextureViewDescriptor::default());

            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("WindowState render encoder"),
                });

            self.shady.add_render_pass(&mut encoder, &view, &[pipeline]);

            self.queue.submit(std::iter::once(encoder.finish()));
            output.present();
        }

        Ok(())
    }

    #[instrument(skip_all)]
    fn update_pipeline(&mut self, shader_source: ShaderSource<'a>) {
        self.pipeline = Some(shady::create_render_pipeline(
            &self.device,
            shader_source,
            &self.config.format,
        ));
    }
}
