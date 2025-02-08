use clap::Parser;
use std::{
    fs::File,
    num::{NonZeroU32, NonZeroUsize},
    time::Duration,
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    terminal::WindowSize,
};
use ratatui::{
    style::{Color, Style},
    widgets::{
        canvas::{Canvas, Line, Shape},
        Bar, BarChart, BarGroup,
    },
    Frame,
};
use shady_audio::{config::ShadyAudioConfig, fetcher::SystemAudioFetcher, ShadyAudio};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(clap::Parser, Debug)]
#[command(version, about)]
struct Ctx {
    /// The bar width
    #[arg(short, long, default_value_t = 3)]
    amount_bars: usize,

    /// The bar color. For a full list of possible colors: https://docs.rs/ratatui/latest/ratatui/style/enum.Color.html
    #[arg(short, long, default_value_t = Color::LightBlue)]
    color: Color,
}

fn main() -> std::io::Result<()> {
    init_logger();

    let mut ctx = Ctx::parse();

    let mut terminal = ratatui::init();
    let mut audio = ShadyAudio::new(
        SystemAudioFetcher::default(|err| panic!("{}", err)),
        ShadyAudioConfig::default(),
    )
    .unwrap();

    loop {
        let window = crossterm::terminal::window_size()?;

        terminal
            .draw(|frame| draw(frame, &mut audio, &ctx))
            .expect("Render frame");

        if event::poll(Duration::from_millis(1000 / 60))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('+') => {
                        ctx.amount_bars += 1;
                        audio.set_bars(NonZeroUsize::new(ctx.amount_bars).unwrap());
                    }
                    KeyCode::Char('-') => {
                        ctx.amount_bars = ctx.amount_bars.saturating_sub(1);
                        audio.set_bars(NonZeroUsize::new(ctx.amount_bars).unwrap());
                    }
                    _ => {}
                }
            }
        }
    }

    ratatui::restore();
    Ok(())
}

fn draw(frame: &mut Frame, audio: &mut ShadyAudio, ctx: &Ctx) {
    const HEIGHT: f64 = 1.;
    const WIDTH: f64 = 1.;

    let bar_values = audio.get_bars();

    let canvas = Canvas::default()
        .x_bounds([0., WIDTH])
        .y_bounds([0., HEIGHT])
        .marker(ratatui::symbols::Marker::HalfBlock)
        .paint(|r_ctx| {
            let slot_width = WIDTH / ctx.amount_bars as f64;
            let gap_width = slot_width / 4.;

            let mut x = 0f64;
            for &bar_value in bar_values {
                r_ctx.draw(&FilledRectangle {
                    x: (x + gap_width) as f64,
                    width: (slot_width / 8.) as f64,
                    height: bar_value as f64,
                    color: Color::Blue,
                });

                x += slot_width;
            }
        });

    frame.render_widget(&canvas, frame.area());
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

struct FilledRectangle {
    x: f64,
    width: f64,
    height: f64,
    color: Color,
}

impl Shape for FilledRectangle {
    fn draw(&self, painter: &mut ratatui::widgets::canvas::Painter) {
        let mut y = 0.;

        while y < self.height {
            Line {
                x1: self.x,
                x2: self.x + self.width,
                y1: y,
                y2: y,
                color: self.color,
            }
            .draw(painter);

            y += 0.001;
        }
    }
}
