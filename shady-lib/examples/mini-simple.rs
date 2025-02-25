/// Every relevant part is marked with the prefix `SHADY` so you can just search in this code with `SHADY`.
use std::{borrow::Cow, sync::Arc};

use pollster::FutureExt;
use shady::{Shady, ShadyDescriptor, ShadyRenderPipeline};
use wgpu::{
    Backends, Device, Instance, Queue, ShaderSource, Surface, SurfaceConfiguration,
    TextureViewDescriptor,
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

struct State<'a> {
    surface: Surface<'a>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    window: Arc<Window>,
    // SHADY
    shady: Shady,
    // SHADY
    pipeline: ShadyRenderPipeline,
}

impl<'a> State<'a> {
    fn new(window: Window) -> Self {
        let window = Arc::new(window);

        let instance = Instance::new(&wgpu::InstanceDescriptor {
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

        let config = {
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

            config
        };

        // SHADY
        //
        // Create the render pipeline which shady will use.
        let pipeline = {
            let fragment_shader = {
                let template = shady::TemplateLang::Wgsl.generate_to_string(None).unwrap();

                ShaderSource::Wgsl(Cow::Owned(template))
            };

            shady::create_render_pipeline(&device, fragment_shader, &config.format)
        };

        // SHADY
        let shady = Shady::new(ShadyDescriptor { device: &device });

        Self {
            surface,
            device,
            queue,
            config,
            window,
            shady,
            pipeline,
        }
    }

    pub fn prepare_next_frame(&mut self) {
        // SHADY
        //
        // Updates the values inside the uniform buffers.
        {
            self.shady.inc_frame();

            self.shady.update_audio_buffer(&mut self.queue);
            self.shady.update_frame_buffer(&mut self.queue);
            self.shady.update_mouse_buffer(&mut self.queue);
            self.shady.update_resolution_buffer(&mut self.queue);
            self.shady.update_time_buffer(&mut self.queue);
        }

        self.surface.configure(&self.device, &self.config);
    }

    pub fn render(&mut self) {
        let output = self.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render encoder"),
            });

        // SHADY
        //
        // Add the render pass to the encoder to draw the next frame.
        self.shady
            .add_render_pass(&mut encoder, &view, std::iter::once(&self.pipeline));

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    pub fn window(&self) -> Arc<Window> {
        self.window.clone()
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        // SHADY
        //
        // Update any properties of shady.
        // Note: You need to call the appropriate `update_*_buffer` method to write
        // the new values into the buffers for the next frame you use shady otherwise the previous values in the
        // buffer will be used.
        self.shady.set_resolution(new_size.width, new_size.height);
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
