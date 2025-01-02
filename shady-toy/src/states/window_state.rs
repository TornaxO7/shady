use std::sync::Arc;

use pollster::FutureExt;
use shady::{ShaderParser, Shady, ShadyDescriptor};
use tracing::trace;
use wgpu::{
    Backends, Device, Instance, Queue, Surface, SurfaceConfiguration, TextureViewDescriptor,
};
use winit::{dpi::PhysicalSize, window::Window};

use super::{SHADY_BIND_GROUP_INDEX, SHADY_VERTEX_BUFFER_INDEX};

use super::RenderState;

pub struct WindowState<'a, S: ShaderParser> {
    surface: Surface<'a>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    window: Arc<Window>,
    pub shady: Shady<S>,
}

impl<'a, S: ShaderParser> WindowState<'a, S> {
    pub fn new(window: Window, fragment_code: &str) -> Result<Self, shady::Error> {
        trace!(
            "Create new WindowState with fragment code:\n{}",
            fragment_code
        );
        let window = Arc::new(window);

        let instance = Instance::new(wgpu::InstanceDescriptor {
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

        let (config, shady) = {
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

            let shady = Shady::new(&ShadyDescriptor {
                device: &device,
                fragment_shader: fragment_code,
                texture_format: surface_format,
                bind_group_index: SHADY_BIND_GROUP_INDEX,
                vertex_buffer_index: SHADY_VERTEX_BUFFER_INDEX,
            })?;

            (config, shady)
        };

        surface.configure(&device, &config);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            window,
            shady,
        })
    }

    pub fn window(&self) -> Arc<Window> {
        self.window.clone()
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.shady
                .update_resolution(new_size.width, new_size.height);
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
}

impl<'a, S: ShaderParser> RenderState<S> for WindowState<'a, S> {
    fn prepare_next_frame(&mut self) {
        self.shady.prepare_next_frame(&mut self.queue);
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("WindowState render encoder"),
            });

        self.shady.add_render_pass(&mut encoder, &view);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn update_pipeline(&mut self, fragment_code: &str) -> Result<(), shady::Error> {
        self.shady
            .update_render_pipeline(&self.device, fragment_code)
    }
}
