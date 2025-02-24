use std::{borrow::Cow, fs::File, io::Read, path::PathBuf};

use ariadne::{Color, Fmt};
use tracing::{debug, warn};
use wgpu::{
    naga::{
        front::{glsl, wgsl},
        ShaderStage,
    },
    ShaderSource, SurfaceError,
};
use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::ActiveEventLoop,
    window::WindowAttributes,
};

use crate::{
    frontend::ShaderLanguage,
    states::{window_state::WindowState, RenderState},
    UserEvent,
};

#[derive(thiserror::Error, Debug)]
enum RenderError {
    #[error(transparent)]
    SurfaceError(#[from] SurfaceError),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error("{0}")]
    WgslParsing(String),

    #[error("{0}")]
    GlslParsing(String),
}

pub struct Renderer<'a> {
    state: Option<WindowState<'a>>,
    display_error: bool,

    shader_lang: ShaderLanguage,

    fragment_path: PathBuf,
}

impl<'a> Renderer<'a> {
    pub fn new(fragment_path: PathBuf, shader_lang: ShaderLanguage) -> anyhow::Result<Self> {
        let mut renderer = Self {
            state: None,
            display_error: true,
            fragment_path,
            shader_lang,
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
        let mut fragment_code = String::new();
        file.read_to_string(&mut fragment_code)?;

        debug!("Fragment code: {}", fragment_code);

        if let Some(state) = &mut self.state {
            let module = match self.shader_lang {
                ShaderLanguage::Wgsl => {
                    debug!("Parsing with wgsl parser");
                    let mut frontend = wgsl::Frontend::new();

                    frontend.parse(&fragment_code).map_err(|err| {
                        RenderError::WgslParsing(err.emit_to_string(&fragment_code))
                    })?
                }
                ShaderLanguage::Glsl => {
                    debug!("Parsing with glsl parser");
                    let mut frontend = glsl::Frontend::default();
                    let options = glsl::Options::from(ShaderStage::Fragment);

                    frontend.parse(&options, &fragment_code).map_err(|err| {
                        RenderError::GlslParsing(err.emit_to_string(&fragment_code))
                    })?
                }
            };

            state.update_pipeline(ShaderSource::Naga(Cow::Owned(module)));
        } else {
            debug!("State not initialized");
        }

        Ok(())
    }
}

impl<'a> ApplicationHandler<UserEvent> for Renderer<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(WindowAttributes::default())
            .unwrap();

        self.state = Some(WindowState::new(window, None));
        self.refresh_fragment_code().unwrap();
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
                    Err(SurfaceError::OutOfMemory) => {
                        unreachable!("Out of memory")
                    }
                    Err(SurfaceError::Timeout) => {
                        warn!("A frame took too long to be present");
                    }
                    Err(err) => warn!("{}", err),
                }
            }
            WindowEvent::Resized(new_size) => state.resize(new_size),
            #[cfg(feature = "mouse")]
            WindowEvent::MouseInput {
                state: mouse_state, ..
            } => {
                let shady = &mut state.shady;
                match mouse_state {
                    winit::event::ElementState::Pressed => {
                        shady.set_mouse_state(shady::MouseState::Pressed)
                    }
                    winit::event::ElementState::Released => {
                        shady.set_mouse_state(shady::MouseState::Released)
                    }
                }
            }
            #[cfg(feature = "mouse")]
            WindowEvent::CursorMoved { position: pos, .. } => {
                state.shady.set_mouse_pos(pos.x as f32, pos.y as f32)
            }
            WindowEvent::KeyboardInput { event, .. }
                if event.logical_key.to_text() == Some("q") =>
            {
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
