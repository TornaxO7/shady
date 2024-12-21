use std::{fs::File, io::Read, path::PathBuf, sync::Arc};

use ariadne::{Color, Fmt, Label, Report, Source};
use pollster::FutureExt;
use shady::{Frontend, Shady};
use tracing::{debug, trace, warn};
use wgpu::{
    Backends, Device, Instance, Queue, RenderPipeline, Surface, SurfaceConfiguration, SurfaceError,
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
    IO(#[from] std::io::Error),
}

struct State<'a, F: Frontend> {
    surface: Surface<'a>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    window: Arc<Window>,
    shady: Shady<F>,
    pipeline: RenderPipeline,

    vbuffer: wgpu::Buffer,
    ibuffer: wgpu::Buffer,
}

impl<'a, F: Frontend> State<'a, F> {
    fn new(window: Window, fragment_code: &str) -> Result<Self, shady::Error> {
        trace!("Create new state with fragment code:\n{}", fragment_code);
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

        let mut shady = Shady::new(&device);

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

        let pipeline = shady.get_render_pipeline(&device, fragment_code, &config.format)?;
        let vbuffer = shady::vertex_buffer(&device);
        let ibuffer = shady::index_buffer(&device);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            window,
            shady,
            pipeline,
            vbuffer,
            ibuffer,
        })
    }

    pub fn prepare_next_frame(&mut self) {
        self.shady.update_buffers(&mut self.queue);

        self.surface.configure(&self.device, &self.config);
    }

    pub fn render(&mut self) -> Result<(), RenderError> {
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

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.shady.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vbuffer.slice(..));
            render_pass.set_index_buffer(self.ibuffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(shady::index_buffer_range(), 0, 0..1);
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
                .update_resolution(new_size.width, new_size.height);
            self.config.width = new_size.width;
            self.config.height = new_size.height;
        }
    }

    pub fn update_pipeline(&mut self, fragment_code: &str) -> Result<(), shady::Error> {
        self.pipeline =
            self.shady
                .get_render_pipeline(&self.device, fragment_code, &self.config.format)?;

        Ok(())
    }
}

pub struct Renderer<'a, F: Frontend> {
    state: Option<State<'a, F>>,
    display_error: bool,

    fragment_path: PathBuf,
    fragment_code: String,
}

impl<'a, F: Frontend> Renderer<'a, F> {
    pub fn new(fragment_path: PathBuf) -> anyhow::Result<Self> {
        let mut renderer = Self {
            state: None,
            display_error: true,
            fragment_path,
            fragment_code: String::new(),
        };

        renderer.refresh_fragment_code()?;
        Ok(renderer)
    }

    fn refresh_fragment_code(&mut self) -> Result<(), RenderError> {
        self.display_error = true;

        debug!(
            "Trying to read from: {}",
            self.fragment_path.to_string_lossy()
        );
        let mut file = File::open(&self.fragment_path)?;
        self.fragment_code.clear();
        file.read_to_string(&mut self.fragment_code)?;

        if let Some(state) = &mut self.state {
            if let Err(err) = state.update_pipeline(&self.fragment_code) {
                self.print_shady_error(err);
            }
        }

        Ok(())
    }

    fn print_shady_error(&mut self, err: shady::Error) {
        self.display_error = false;
        match err {
            shady::Error::InvalidWgslFragmentShader {
                msg,
                fragment_code,
                offset,
                length,
                ..
            } => {
                Report::build(
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
                .unwrap();
            }

            shady::Error::InvalidGlslFragmentShader(msg) => {
                eprintln!("{}", msg);
            }
        };
    }
}

impl<'a, F: Frontend> ApplicationHandler<UserEvent> for Renderer<'a, F> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(WindowAttributes::default())
            .unwrap();

        match State::<F>::new(window, &self.fragment_code) {
            Ok(state) => self.state = Some(state),
            Err(err) => self.print_shady_error(err),
        }
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

                match state.render() {
                    Ok(_) => {
                        if !self.display_error {
                            println!("[{}] Everything clear", "OK".fg(Color::Green));
                            self.display_error = true;
                        }
                    }
                    Err(RenderError::SurfaceError(SurfaceError::OutOfMemory)) => {
                        unreachable!("Out of memory")
                    }
                    Err(RenderError::SurfaceError(SurfaceError::Timeout)) => {
                        warn!("A frame took too long to be present");
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
                if let Err(err) = self.refresh_fragment_code() {
                    eprintln!("Couldn't refresh fragment code: {}", err);
                }
            }
        }
    }
}
