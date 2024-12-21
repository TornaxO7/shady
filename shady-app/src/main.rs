mod cli;
mod frontend;
mod logger;
mod renderer;

use std::{
    path::{Path, PathBuf},
    sync::{mpsc, Arc},
};

use anyhow::Result;
use ariadne::Fmt;
use frontend::Frontend;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use renderer::Renderer;
use tracing::{debug, debug_span};
use winit::{
    error::EventLoopError,
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
};

pub const WGSL_TEMPLATE: &str = include_str!("template.wgsl");
pub const GLSL_TEMPLATE: &str = include_str!("template.glsl");

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("File does not exist")]
    FileDoesNotExist,

    #[error("{0} isn't a (shader-)file.")]
    NoShaderfile(String),

    #[error(transparent)]
    WinitEventLoop(#[from] EventLoopError),

    #[error(transparent)]
    FileWatcher(#[from] notify::Error),

    #[error("{0}")]
    UnknownShaderFileExtension(String),
}

#[derive(Debug, Clone, Copy)]
enum UserEvent {
    UpdatePath,
}

fn main() -> Result<()> {
    logger::init();
    let args = cli::parse();

    if args.template {
        add_template_to_file(&args.fragment_path)
            .map_err(|err| Error::UnknownShaderFileExtension(err))?;
    }

    if !std::fs::exists(&args.fragment_path).expect("Check if fragment file exists") {
        eprintln!(
            "The given fragment path does not exist: \"{}\"",
            args.fragment_path.to_string_lossy()
        );
        std::process::exit(1);
    }

    let frontend = Frontend::try_from(args.fragment_path.as_path())
        .map_err(|err| Error::UnknownShaderFileExtension(err))?;

    println!(
        "[{}]: Press `q` in the shader-window to exit.",
        "NOTE".fg(ariadne::Color::Cyan)
    );

    start_app(args.fragment_path, frontend)
}

fn start_app(fragment_path: PathBuf, frontend: Frontend) -> Result<()> {
    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .build()
        .expect("Create window eventloop");
    event_loop.set_control_flow(ControlFlow::Wait);

    let proxy = Arc::new(event_loop.create_proxy());

    std::thread::spawn({
        let path = fragment_path.clone();
        move || watch_shader_file(path, proxy)
    });

    let mut renderer = Renderer::new(fragment_path, frontend)?;
    event_loop.run_app(&mut renderer)?;

    Ok(())
}

fn watch_shader_file<P: AsRef<Path>>(path: P, proxy: Arc<EventLoopProxy<UserEvent>>) -> Result<()> {
    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
    let mut watcher = notify::recommended_watcher(tx)?;
    let span = debug_span!("Watcher");
    let _enter = span.enter();

    watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;

    for res in rx {
        match res {
            Ok(event) => {
                debug!("Event: {:?}", event);
                match event.kind {
                    EventKind::Remove(_) => {
                        watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;
                    }
                    EventKind::Modify(_) => {
                        // we wait some time first, since some editors might quickly remove it and reinsert the file
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        proxy.send_event(UserEvent::UpdatePath)?
                    }
                    _ => (),
                };
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}

fn add_template_to_file(path: &Path) -> Result<(), String> {
    let frontend = Frontend::try_from(path)?;

    match frontend {
        Frontend::Wgsl => std::fs::write(path, WGSL_TEMPLATE),
        Frontend::Glsl => std::fs::write(path, GLSL_TEMPLATE),
    }
    .expect("Write template to given path");

    Ok(())
}
