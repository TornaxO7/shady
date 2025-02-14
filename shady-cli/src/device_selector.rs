use crossterm::{
    event::{Event, KeyCode, KeyEvent},
    terminal::WindowSize,
};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Style},
    widgets::{Block, List, ListState},
};
use shady_audio::cpal::{
    self,
    traits::{DeviceTrait, HostTrait},
    Device, SampleFormat,
};
use tracing::warn;

use crate::{Action, Model};

pub struct DeviceChooser {
    device_names: Vec<String>,
    list_state: ListState,
}

impl DeviceChooser {
    pub fn boxed() -> Box<Self> {
        let host = cpal::default_host();

        let output_devices = match host.output_devices() {
            Ok(output_devices) => output_devices
                .filter(device_condition)
                .map(|device| device.name().unwrap())
                .collect(),
            Err(err) => {
                warn!("Couldn't retrieve output devices from host: {}", err);
                Vec::with_capacity(0)
            }
        };

        let output_state = ListState::default();

        Box::new(Self {
            device_names: output_devices,
            list_state: output_state,
        })
    }

    fn selected(&self) -> Option<String> {
        self.list_state
            .selected()
            .map(|name| self.device_names[name].clone())
    }
}

impl Model for DeviceChooser {
    fn draw(&mut self, frame: &mut ratatui::Frame, _: WindowSize) {
        let column = {
            let column = Layout::horizontal([
                Constraint::Percentage(25),
                Constraint::Percentage(50),
                Constraint::Percentage(25),
            ])
            .split(frame.area())[1];

            Layout::vertical([
                Constraint::Percentage(25),
                Constraint::Percentage(50),
                Constraint::Percentage(25),
            ])
            .split(column)[1]
        };

        let layout = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(column);

        let output_devices_list = List::new(self.device_names.clone())
            .block(Block::bordered().title("Select an audio device for the audio source"))
            .highlight_symbol(">> ")
            .highlight_style(Style::new().fg(Color::Blue));

        frame.render_stateful_widget(output_devices_list, layout[0], &mut self.list_state);
    }

    fn handle_event(&mut self, event: Event) -> Action {
        if let Event::Key(KeyEvent { code, .. }) = event {
            match code {
                KeyCode::Char('q') => return Action::Quit,
                KeyCode::Char('j') => self.list_state.select_next(),
                KeyCode::Char('k') => self.list_state.select_previous(),
                KeyCode::Enter => {
                    if let Some(device_name) = self.selected() {
                        return Action::StartVisualizer { device_name };
                    }
                }
                _ => {}
            };
        }

        Action::None
    }
}

fn device_condition(device: &Device) -> bool {
    if device.name().is_err() {
        return false;
    }

    match device.supported_output_configs() {
        Ok(mut configs) => configs.any(|config| config.sample_format() == SampleFormat::F32),
        Err(_) => false,
    }
}
