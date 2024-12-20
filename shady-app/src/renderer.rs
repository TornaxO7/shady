use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use ariadne::{Color, Fmt, Label, Report, Source};
use pollster::FutureExt;
use shady::Shady;
use tracing::warn;
use wgpu::{
    Backends, Device, Instance, Queue, Surface, SurfaceConfiguration, SurfaceError,
    TextureViewDescriptor,
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes},
};

use crate::UserEvent;

#[derive(thiserror::Error, Debug)]
enum RenderError {
    #[error(transparent)]
    SurfaceError(#[from] SurfaceError),

    #[error(transparent)]
    Shady(#[from] shady::Error),
}

struct State<'a> {
    surface: Surface<'a>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    window: Arc<Window>,
    shady: Shady,
}

impl<'a> State<'a> {
    fn new(window: Window) -> Self {
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
        self.shady.update_buffers(&mut self.queue);

        self.surface.configure(&self.device, &self.config);
    }

    pub fn render<P: AsRef<Path>>(&mut self, fragment_path: P) -> Result<(), RenderError> {
        let pipeline = {
            let fragment_shader =
                std::fs::read_to_string(fragment_path.as_ref()).expect("Read fragment shader");

            self.shady
                .get_render_pipeline(&self.device, fragment_shader, &self.config.format)
        }?;

        let output = self.surface.get_current_texture()?;
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
            render_pass.set_bind_group(0, self.shady.bind_group(), &[]);
            render_pass.set_vertex_buffer(0, self.shady.vbuffer.slice(..));
            render_pass.set_index_buffer(self.shady.ibuffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(self.shady.ibuffer_range(), 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn cleanup(&mut self) {
        self.shady.cleanup();
    }

    pub fn window(&self) -> Arc<Window> {
        self.window.clone()
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.shady
                .update_resolution(new_size.width as f32, new_size.height as f32);
            self.config.width = new_size.width;
            self.config.height = new_size.height;
        }
    }
}

pub struct Renderer<'a> {
    state: Option<State<'a>>,
    fragment_path: PathBuf,
    display_error: bool,
}

impl<'a> Renderer<'a> {
    pub fn new(fragment_path: PathBuf) -> Self {
        Self {
            state: None,
            fragment_path,
            display_error: true,
        }
    }
}

impl<'a> ApplicationHandler<UserEvent> for Renderer<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(WindowAttributes::default())
            .unwrap();

        self.state = Some(State::new(window));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = &mut self.state else { return };
        let window = state.window();

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                window.request_redraw();
                state.prepare_next_frame();

                match state.render(&self.fragment_path) {
                    Ok(_) => {
                        if !self.display_error {
                            println!("[{}] Everything clear", "OK".fg(Color::Green));
                        }
                        self.display_error = true;
                    }
                    Err(RenderError::SurfaceError(SurfaceError::OutOfMemory)) => {
                        unreachable!("Out of memory")
                    }
                    Err(RenderError::SurfaceError(SurfaceError::Timeout)) => {
                        warn!("A frame took too long to be present");
                    }
                    Err(RenderError::Shady(err)) => {
                        if self.display_error {
                            self.display_error = false;
                            match err {
                                shady::Error::InvalidFragmentShader {
                                    msg,
                                    fragment_code,
                                    offset,
                                    length,
                                    ..
                                } => Report::build(
                                    ariadne::ReportKind::Error,
                                    offset as usize..(offset + length) as usize,
                                )
                                .with_message("Invalid fragment shader")
                                .with_label(
                                    Label::new(offset as usize..(offset + length) as usize)
                                        .with_message(msg.fg(ariadne::Color::Blue))
                                        .with_color(ariadne::Color::Blue),
                                )
                                .finish()
                                .print(Source::from(fragment_code))
                                .unwrap(),
                            };
                        }
                    }
                    Err(err) => warn!("{}", err),
                }
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

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::UpdatePath => {
                if let Some(state) = &mut self.state {
                    state.prepare_next_frame();
                }
                self.display_error = true;
            }
        }
    }
}
