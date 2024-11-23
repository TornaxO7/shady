/// Every relevant part is marked with the prefix `SHADY` so you can just search in this code with `SHADY`.
use std::sync::Arc;

use pollster::FutureExt;
use shady::Shady;
use wgpu::{
    Backends, Device, Instance, Queue, Surface, SurfaceConfiguration, TextureViewDescriptor,
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

const FRAGMENT_SHADER: &str = include_str!("../src/template.wgsl");

struct State<'a> {
    surface: Surface<'a>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    window: Arc<Window>,
    // SHADY
    shady: Shady,
}

impl<'a> State<'a> {
    fn new(window: Window) -> Self {
        let window = Arc::new(window);

        let instance = Instance::new(wgpu::InstanceDescriptor {
            backends: Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .block_on()
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .block_on()
            .unwrap();

        // SHADY
        let shady = Shady::new(&device);

        let config = {
            let surface_caps = surface.get_capabilities(&adapter);
            let surface_format = surface_caps
                .formats
                .iter()
                .find(|f| f.is_srgb())
                .copied()
                .unwrap_or(surface_caps.formats[0]);

            let size = window.clone().inner_size();

            wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width: size.width,
                height: size.height,
                present_mode: wgpu::PresentMode::AutoVsync,
                alpha_mode: surface_caps.alpha_modes[0],
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            }
        };

        Self {
            surface,
            device,
            queue,
            config,
            window,
            shady,
        }
    }

    pub fn prepare_next_frame(&mut self) {
        // SHADY
        self.shady.update_buffers(&mut self.queue);

        self.surface.configure(&self.device, &self.config);
    }

    pub fn render(&mut self) {
        // SHADY
        let pipeline = self
            .shady
            .get_render_pipeline(&self.device, FRAGMENT_SHADER, &self.config.format)
            .unwrap();

        let output = self.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });

            render_pass.set_pipeline(&pipeline);

            // SHADY
            render_pass.set_bind_group(0, self.shady.bind_group(), &[]);
            render_pass.set_vertex_buffer(0, self.shady.vbuffer.slice(..));
            render_pass.set_index_buffer(self.shady.ibuffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(self.shady.ibuffer_range(), 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    pub fn window(&self) -> Arc<Window> {
        self.window.clone()
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        // SHADY
        self.shady
            .update_resolution(new_size.width as f32, new_size.height as f32);
    }

    pub fn cleanup(&mut self) {
        // SHADY
        self.shady.cleanup();
    }
}

struct App<'a> {
    state: Option<State<'a>>,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        Self { state: None }
    }
}

impl<'a> ApplicationHandler<()> for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(WindowAttributes::default())
            .unwrap();

        self.state = Some(State::new(window));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = &mut self.state else { return };
        let window = state.window();

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                window.request_redraw();
                state.prepare_next_frame();
                state.render();
            }
            WindowEvent::Resized(new_size) => state.resize(new_size),
            WindowEvent::KeyboardInput { event, .. }
                if event.logical_key.to_text() == Some("q") =>
            {
                if let Some(state) = &mut self.state {
                    state.cleanup();
                }

                event_loop.exit();
            }
            _ => (),
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new();

    event_loop.run_app(&mut app).unwrap();
}
