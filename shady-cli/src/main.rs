mod device_selector;
mod visualizer;

use clap::Parser;
use device_selector::DeviceChooser;
use std::{fs::File, time::Duration};
use tracing::debug;
use visualizer::Visualizer;

use crossterm::{
    event::{self, Event},
    terminal::WindowSize,
};
use ratatui::{style::Color, Frame};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(clap::Parser, Debug)]
#[command(version, about)]
struct Cli {
    /// The bar color. For a full list of possible colors: https://docs.rs/ratatui/latest/ratatui/style/enum.Color.html
    #[arg(short, long, default_value_t = Color::LightBlue)]
    color: Color,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Action {
    StartVisualizer {
        device_name: String,
        is_output_device: bool,
    },
    StartDeviceMenu,
    Quit,
    None,
}

trait Model {
    fn draw(&mut self, frame: &mut Frame, window_size: WindowSize);

    fn handle_event(&mut self, event: Event) -> Action;
}

fn main() -> std::io::Result<()> {
    init_logger();

    let cli = Cli::parse();
    let mut terminal = ratatui::init();
    let mut action = Action::None;
    let mut model: Box<dyn Model> = DeviceChooser::boxed();

    loop {
        let window_size = crossterm::terminal::window_size()?;
        terminal.draw(|frame| model.draw(frame, window_size))?;

        if event::poll(Duration::from_millis(1000 / 60))? {
            action = model.handle_event(event::read()?);
        }

        match action.clone() {
            Action::StartVisualizer {
                device_name,
                is_output_device,
            } => {
                debug!(
                    "Selected device: '{}'. Is output device: {}",
                    device_name, is_output_device
                );

                match Visualizer::boxed(device_name, is_output_device) {
                    Ok(visualizer) => model = visualizer,
                    Err(err) => {
                        tracing::error!("Couldn't start visualizer: {}", err);
                        action = Action::StartDeviceMenu;
                    }
                }
            }
            Action::StartDeviceMenu => {
                model = DeviceChooser::boxed();
                action = Action::StartDeviceMenu;
            }
            Action::Quit => break,
            Action::None => {}
        }
    }

    ratatui::restore();
    Ok(())
}

fn init_logger() {
    let file = File::create("/tmp/shady-cli.log").unwrap();

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_writer(file)
        .without_time();

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(EnvFilter::from_env(EnvFilter::DEFAULT_ENV))
        .init();
}
