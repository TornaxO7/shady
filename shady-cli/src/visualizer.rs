use std::num::NonZeroUsize;

use crossterm::{
    event::{Event, KeyCode, KeyEvent},
    terminal::WindowSize,
};
use ratatui::{
    style::{Color, Style},
    widgets::{Bar, BarChart, BarGroup},
};
use shady_audio::{
    config::ShadyAudioConfig,
    cpal::{
        self,
        traits::{DeviceTrait, HostTrait},
        SampleFormat,
    },
    fetcher::SystemAudioFetcher,
    ShadyAudio,
};

use crate::{Action, Model};

const HEIGHT: u64 = 1_000;

pub struct Visualizer<'a> {
    bar_width: u16,
    bars: Vec<Bar<'a>>,

    prev_terminal_columns: u16,

    audio: ShadyAudio,
}

impl<'a> Visualizer<'a> {
    pub fn boxed(device_name: String, is_output_device: bool) -> Result<Box<Self>, String> {
        let device = {
            let host = cpal::default_host();

            if is_output_device {
                host.output_devices()
                    .unwrap()
                    .find(|device| match device.name() {
                        Ok(name) => name == device_name,
                        Err(_) => false,
                    })
                    .unwrap()
            } else {
                host.input_devices()
                    .unwrap()
                    .find(|device| match device.name() {
                        Ok(name) => name == device_name,
                        Err(_) => false,
                    })
                    .unwrap()
            }
        };

        let config = {
            let Ok(supported_output_configs) = device.supported_output_configs() else {
                todo!();
            };

            let mut supported_output_configs: Vec<_> = supported_output_configs
                .filter(|entry| entry.sample_format() == SampleFormat::F32)
                .collect();

            supported_output_configs.sort_by(|a, b| a.cmp_default_heuristics(b));
            let Some(config) = supported_output_configs.into_iter().next() else {
                todo!();
            };

            config
        };

        let Ok(fetcher) = SystemAudioFetcher::new(&device, &config, |err| panic!("{}", err)) else {
            todo!()
        };

        let Ok(audio) = ShadyAudio::new(fetcher, ShadyAudioConfig::default()) else {
            todo!()
        };

        Ok(Box::new(Self {
            bar_width: 3,
            prev_terminal_columns: 0,
            bars: Vec::new(),
            audio,
        }))
    }

    fn set_bars(&mut self, columns: u16) {
        let amount_bars = self.amount_bars(columns);

        self.bars.resize(
            usize::from(amount_bars),
            Bar::default().text_value("".into()),
        );

        self.audio.set_bars(amount_bars);
    }

    fn get_bars(&mut self) -> &[Bar<'a>] {
        let bar_values = self.audio.get_bars();

        for (&value, bar) in bar_values.iter().zip(self.bars.iter_mut()) {
            *bar = bar.clone().value((HEIGHT as f32 * value) as u64);
        }

        self.bars.as_slice()
    }

    fn amount_bars(&self, columns: u16) -> NonZeroUsize {
        NonZeroUsize::new((columns / self.bar_width) as usize).unwrap()
    }
}

impl<'a> Model for Visualizer<'a> {
    fn draw(&mut self, frame: &mut ratatui::Frame, window_size: WindowSize) {
        if window_size.columns != self.prev_terminal_columns {
            self.prev_terminal_columns = window_size.columns;
            self.set_bars(window_size.columns);
        }

        let bar_chart = BarChart::default()
            .bar_width(self.bar_width)
            .bar_gap(1)
            .bar_style(Style::new().fg(Color::Blue))
            .data(BarGroup::default().label("".into()).bars(self.get_bars()))
            .max(HEIGHT);

        frame.render_widget(&bar_chart, frame.area());
    }

    fn handle_event(&mut self, event: crossterm::event::Event) -> Action {
        if let Event::Key(KeyEvent { code, .. }) = event {
            match code {
                KeyCode::Char('q') => return Action::Quit,
                KeyCode::Char('d') => return Action::StartDeviceMenu,
                _ => {}
            }
        }

        Action::None
    }
}
