use clap::Parser;
use std::{fs::File, num::NonZeroUsize, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{
    style::Color,
    widgets::{Bar, BarChart, BarGroup},
    Frame,
};
use shady_audio::{config::ShadyAudioConfig, fetcher::SystemAudioFetcher, ShadyAudio};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(clap::Parser, Debug)]
#[command(version, about)]
struct Ctx {
    /// The bar width
    #[arg(short, long, default_value_t = 3)]
    bar_width: u16,

    /// The bar color. For a full list of possible colors: https://docs.rs/ratatui/latest/ratatui/style/enum.Color.html
    #[arg(short, long, default_value_t = Color::LightBlue)]
    color: Color,
}

impl Ctx {
    fn amount_bars(&self, columns: u16) -> NonZeroUsize {
        NonZeroUsize::new((columns / self.bar_width) as usize).unwrap()
    }
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

    let mut prev_columns = 0;
    loop {
        let window_size = crossterm::terminal::window_size()?;
        if prev_columns != window_size.columns {
            prev_columns = window_size.columns;
            audio.set_bars(ctx.amount_bars(window_size.columns));
        }

        terminal
            .draw(|frame| draw(frame, &mut audio, &ctx))
            .expect("Render frame");

        if event::poll(Duration::from_millis(1000 / 60))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('+') => {
                        ctx.bar_width += 1;
                        audio.set_bars(ctx.amount_bars(window_size.columns));
                    }
                    KeyCode::Char('-') => {
                        ctx.bar_width = 1.max(ctx.bar_width - 1);
                        audio.set_bars(ctx.amount_bars(window_size.columns));
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
    const HEIGHT: u64 = 1000;

    let bar_values = audio.get_bars();
    let bars = bar_values
        .iter()
        .map(|&value| {
            Bar::default()
                .value((HEIGHT as f32 * value) as u64)
                .text_value("".to_string())
        })
        .collect::<Vec<Bar>>();

    let bar_chart = BarChart::default()
        .bar_width(ctx.bar_width)
        .bar_gap(1)
        .data(BarGroup::default().label("".into()).bars(&bars))
        .max(HEIGHT);

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
