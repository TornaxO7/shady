use clap::Parser;
use std::{fs::File, num::NonZeroUsize, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{
    style::{Color, Style},
    widgets::{Bar, BarChart, BarGroup},
    Frame,
};
use shady_audio::{config::EqualizerConfig, fetcher::SystemAudioFetcher, ShadyAudio};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

const HEIGHT: u64 = 1000;

#[derive(clap::Parser, Debug)]
#[command(version, about)]
struct Cli {
    /// The bar color. For a full list of possible colors: https://docs.rs/ratatui/latest/ratatui/style/enum.Color.html
    #[arg(short, long, default_value_t = Color::LightBlue)]
    color: Color,
}

struct Ctx<'a> {
    bar_width: u16,
    bars: Vec<Bar<'a>>,
    color: Color,

    audio: ShadyAudio,
}

impl<'a> Ctx<'a> {
    fn amount_bars(&self, columns: u16) -> NonZeroUsize {
        NonZeroUsize::new((columns / self.bar_width) as usize).unwrap()
    }

    fn set_bars(&mut self, columns: u16) {
        let amount_bars = self.amount_bars(columns);

        self.bars.resize(
            usize::from(amount_bars),
            Bar::default().text_value("".to_string()),
        );

        self.audio.set_bars(amount_bars);
    }

    fn get_bars(&mut self) -> &[Bar<'a>] {
        let bar_values = self.audio.get_bars();

        for (value, bar) in bar_values.iter().zip(self.bars.iter_mut()) {
            *bar = bar.clone().value((HEIGHT as f32 * value) as u64);
        }

        self.bars.as_slice()
    }
}

fn main() -> std::io::Result<()> {
    init_logger();

    let cli = Cli::parse();
    let mut ctx = Ctx {
        bar_width: 3,
        bars: Vec::new(),
        color: cli.color,
        audio: ShadyAudio::new(
            SystemAudioFetcher::default(|err| panic!("{}", err)).unwrap(),
            EqualizerConfig::default(),
        )
        .unwrap(),
    };

    let mut terminal = ratatui::init();

    let mut prev_columns = 0;
    loop {
        let window_size = crossterm::terminal::window_size()?;
        if prev_columns != window_size.columns {
            prev_columns = window_size.columns;
            ctx.set_bars(window_size.columns);
        }

        terminal
            .draw(|frame| draw(frame, &mut ctx))
            .expect("Render frame");

        if event::poll(Duration::from_millis(1000 / 60))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('+') => {
                        ctx.bar_width += 1;
                        ctx.set_bars(window_size.columns);
                    }
                    KeyCode::Char('-') => {
                        ctx.bar_width = 1.max(ctx.bar_width - 1);
                        ctx.set_bars(window_size.columns);
                    }
                    _ => {}
                }
            }
        }
    }

    ratatui::restore();
    Ok(())
}

fn draw(frame: &mut Frame, ctx: &mut Ctx) {
    let bar_chart = BarChart::default()
        .bar_width(ctx.bar_width)
        .bar_gap(1)
        .bar_style(Style::new().fg(ctx.color))
        .data(BarGroup::default().label("".into()).bars(ctx.get_bars()))
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
