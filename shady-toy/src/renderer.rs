use std::{borrow::Cow, fs::File, io::Read, path::PathBuf};

use ariadne::{Color, Fmt, Label, Report, Source};
use shady::MouseState;
use tracing::{debug, warn};
use wgpu::{
    naga::{FastHashMap, ShaderStage},
    ShaderSource, SurfaceError,
};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, WindowEvent},
    event_loop::ActiveEventLoop,
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

        let cow_code = Cow::Owned(fragment_code);

        if let Some(state) = &mut self.state {
            let shader_source = match self.shader_lang {
                ShaderLanguage::Wgsl => ShaderSource::Wgsl(cow_code),
                ShaderLanguage::Glsl => ShaderSource::Glsl {
                    shader: cow_code,
                    stage: ShaderStage::Fragment,
                    defines: FastHashMap::default(),
                },
            };

            state.update_pipeline(shader_source);
        }

        Ok(())
    }

    fn _print_shady_error(&mut self, err: shady::Error) {
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

impl<'a> ApplicationHandler<UserEvent> for Renderer<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(WindowAttributes::default())
            .unwrap();

        self.state = Some(WindowState::new(window, None));
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
            WindowEvent::MouseInput {
                state: mouse_state, ..
            } => {
                let shady = &mut state.shady;
                match mouse_state {
                    ElementState::Pressed => shady.update_mouse_input(MouseState::Pressed),
                    ElementState::Released => shady.update_mouse_input(MouseState::Released),
                }
            }
            WindowEvent::CursorMoved { position: pos, .. } => {
                state.shady.update_cursor(pos.x as f32, pos.y as f32)
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
