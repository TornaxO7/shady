use clap::Parser;
use std::{fs::File, num::NonZero, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{
    style::{Color, Style},
    widgets::{Bar, BarChart, BarGroup},
    Frame,
};
use shady_audio::{
    fetcher::{SystemAudioFetcher, SystemAudioFetcherDescriptor},
    util::DeviceType,
    BarProcessor, BarProcessorConfig, InterpolationVariant, SampleProcessor,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

const HEIGHT: u64 = 1000;

#[derive(clap::Parser, Debug)]
#[command(version, about)]
struct Cli {
    /// The bar color. For a full list of possible colors: https://docs.rs/ratatui/latest/ratatui/style/enum.Color.html
    #[arg(short, long, default_value_t = Color::LightBlue)]
    color: Color,

    /// If `shady-cli` should print all available output devices which you can
    /// pass to `--output_device`
    #[arg(long)]
    pub show_output_devices: bool,

    /// Choose the output device `shady-cli` should use. You can get a list of devices by invoking `shady-cli` with the `--show-output-devices` argument.
    #[arg(long)]
    pub output_device: Option<String>,
}

struct Ctx<'a> {
    bar_width: u16,
    bars: Vec<Bar<'a>>,
    color: Color,

    sample_processor: SampleProcessor,
    bar_processor: BarProcessor,
    interpolation: InterpolationVariant,
}

impl<'a> Ctx<'a> {
    fn amount_bars(&self, columns: u16) -> NonZero<u16> {
        NonZero::new(columns / self.bar_width).unwrap()
    }

    fn set_bars(&mut self, columns: u16) {
        let amount_bars = self.amount_bars(columns);

        self.bars.resize(
            usize::from(u16::from(amount_bars)),
            Bar::default().text_value("".to_string()),
        );

        self.bar_processor = BarProcessor::new(
            &self.sample_processor,
            BarProcessorConfig {
                amount_bars,
                ..self.bar_processor.config().clone()
            },
        );
    }

    fn get_bars(&mut self) -> &[Bar<'a>] {
        self.sample_processor.process_next_samples();
        let bar_values = self.bar_processor.process_bars(&self.sample_processor);

        for (value, bar) in bar_values.iter().zip(self.bars.iter_mut()) {
            *bar = bar.clone().value((HEIGHT as f32 * value) as u64);
        }

        self.bars.as_slice()
    }

    fn next_interpolation(&mut self) {
        self.interpolation = match self.interpolation {
            InterpolationVariant::None => InterpolationVariant::Linear,
            InterpolationVariant::Linear => InterpolationVariant::CubicSpline,
            InterpolationVariant::CubicSpline => InterpolationVariant::None,
        };

        self.bar_processor = BarProcessor::new(
            &self.sample_processor,
            BarProcessorConfig {
                interpolation: self.interpolation,
                ..self.bar_processor.config().clone()
            },
        );
    }
}

fn main() -> std::io::Result<()> {
    init_logger();

    let cli = Cli::parse();
    if cli.show_output_devices {
        print_available_output_devices();
        println!("Choose one of them and add it to the cli as an argument.");
        return Ok(());
    }

    let mut ctx = {
        let device = match cli.output_device {
            Some(device_name) => {
                match shady_audio::util::get_device(&device_name, DeviceType::Output)
                    .expect("Host has output devices")
                {
                    Some(device) => device,
                    None => {
                        print_available_output_devices();
                        panic!(
                            "There isn't an output device called: \"{}\".\nChoose another one.",
                            &device_name
                        );
                    }
                }
            }
            None => shady_audio::util::get_default_device(DeviceType::Output)
                .expect("Default output device exists"),
        };

        let descriptor = SystemAudioFetcherDescriptor {
            device,
            ..Default::default()
        };

        let sample_processor = SampleProcessor::new(SystemAudioFetcher::new(&descriptor).unwrap());
        let bar_processor = BarProcessor::new(&sample_processor, BarProcessorConfig::default());

        Ctx {
            bar_width: 3,
            bars: Vec::new(),
            color: cli.color,
            sample_processor,
            bar_processor,
            interpolation: InterpolationVariant::CubicSpline,
        }
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
                    KeyCode::Char('i') => {
                        ctx.next_interpolation();
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

fn print_available_output_devices() {
    let names = shady_audio::util::get_device_names(DeviceType::Output)
        .expect("Host has audio output devices");

    println!("======\nAvailable output devices:\n{:#?}", names);
}
