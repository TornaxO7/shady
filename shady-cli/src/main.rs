use clap::Parser;
use std::{fs::File, time::Duration};

use crossterm::{
    event::{self, Event},
    terminal::WindowSize,
};
use ratatui::{
    style::{Color, Style},
    widgets::{Bar, BarChart, BarGroup},
    Frame,
};
use shady_audio::{config::ShadyAudioConfig, fetcher::SystemAudioFetcher, ShadyAudio};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(clap::Parser, Debug)]
#[command(version, about)]
struct Ctx {
    /// The bar width
    #[arg(short, long, default_value_t = 1)]
    bar_width: u16,

    /// The bar color. For a full list of possible colors: https://docs.rs/ratatui/latest/ratatui/style/enum.Color.html
    #[arg(short, long, default_value_t = Color::LightBlue)]
    color: Color,
}

fn main() -> std::io::Result<()> {
    init_logger();

    let ctx = Ctx::parse();

    let mut terminal = ratatui::init();
    let mut audio = ShadyAudio::new(
        SystemAudioFetcher::default(|err| panic!("{}", err)),
        ShadyAudioConfig::default(),
    );

    loop {
        let window_size = crossterm::terminal::window_size()?;

        terminal
            .draw(|frame| draw(frame, &mut audio, window_size, &ctx))
            .expect("Render frame");

        if event::poll(Duration::from_millis(1000 / 60))? {
            if matches!(event::read()?, Event::Key(_)) {
                break;
            }
        }
    }

    ratatui::restore();
    Ok(())
}

fn draw(frame: &mut Frame, audio: &mut ShadyAudio, window_size: WindowSize, ctx: &Ctx) {
    const MAX_HEIGHT: u64 = 100;

    let bar_group = {
        let spline = audio.get_spline();
        let mut bars = Vec::with_capacity(window_size.columns.into());
        for column in 0..window_size.columns / ctx.bar_width {
            let frac = (column as f32) / (window_size.columns as f32 / ctx.bar_width as f32);

            let value = spline.sample(frac).unwrap_or(0.0);
            bars.push(
                Bar::default()
                    .text_value("".to_string())
                    .value((value * MAX_HEIGHT as f32) as u64),
            );
        }

        BarGroup::default().label("".into()).bars(&bars)
    };

    let bar_chart = BarChart::default()
        .bar_style(Style::new())
        .data(bar_group)
        .bar_width(ctx.bar_width)
        .bar_gap(1)
        .bar_style(Style::new().fg(ctx.color))
        .max(MAX_HEIGHT);

    frame.render_widget(&bar_chart, frame.area());
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
