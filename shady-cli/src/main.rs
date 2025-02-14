mod device_selector;

use clap::Parser;
use device_selector::DeviceChooser;
use std::{fs::File, num::NonZeroUsize, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{
    style::{Color, Style},
    widgets::{Bar, BarChart, BarGroup},
    Frame,
};
use shady_audio::{config::ShadyAudioConfig, fetcher::SystemAudioFetcher, ShadyAudio};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

const HEIGHT: u64 = 1000;

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
    Quit,
    None,
}

trait Model {
    fn draw(&mut self, frame: &mut Frame);

    fn handle_event(&mut self, event: Event) -> Action;
}

fn main() -> std::io::Result<()> {
    init_logger();

    let cli = Cli::parse();
    let mut terminal = ratatui::init();
    let mut action = Action::None;
    let mut model = DeviceChooser::boxed();

    loop {
        terminal.draw(|frame| model.draw(frame))?;

        if event::poll(Duration::from_millis(1000 / 60))? {
            action = model.handle_event(event::read()?);
        }

        match action.clone() {
            Action::StartVisualizer {
                device_name,
                is_output_device,
            } => {
                tracing::error!("{}", device_name);
                break;
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
