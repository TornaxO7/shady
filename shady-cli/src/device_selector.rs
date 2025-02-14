use crossterm::{
    event::{Event, KeyCode, KeyEvent},
    terminal::WindowSize,
};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::Line,
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
    input_devices: Vec<String>,
    output_devices: Vec<String>,

    input_state: ListState,
    output_state: ListState,
}

impl DeviceChooser {
    pub fn boxed() -> Box<Self> {
        let host = cpal::default_host();

        let input_devices = match host.input_devices() {
            Ok(input_devices) => input_devices
                .filter(device_condition)
                .map(|device| device.name().unwrap())
                .collect(),
            Err(err) => {
                warn!("Couldn't retrieve input devices from host: {}", err);
                Vec::with_capacity(0)
            }
        };

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

        let input_state = ListState::default();
        let output_state = ListState::default();

        Box::new(Self {
            input_devices,
            output_devices,
            input_state,
            output_state,
        })
    }

    fn select_next(&mut self) {
        let is_in_input_list = self.input_state.selected().is_some();
        let is_in_output_list = self.output_state.selected().is_some();
        let is_at_bottom_from_input_list = self
            .input_state
            .selected()
            .map(|index| index == self.input_devices.len() - 1)
            .unwrap_or(false);
        let is_at_bottom_from_output_list = self
            .output_state
            .selected()
            .map(|index| index == self.output_devices.len() - 1)
            .unwrap_or(false);

        if is_in_output_list {
            if is_at_bottom_from_output_list {
                self.output_state.select(None);
                self.input_state.select_first();
            } else {
                self.output_state.select_next();
            }
        } else if is_in_input_list {
            if is_at_bottom_from_input_list {
                self.output_state.select_first();
                self.input_state.select(None);
            } else {
                self.input_state.select_next();
            }
        } else {
            self.output_state.select_first();
        }
    }

    fn select_previous(&mut self) {
        let is_in_input_list = self.input_state.selected().is_some();
        let is_in_output_list = self.output_state.selected().is_some();
        let is_at_top_from_input_list = self
            .input_state
            .selected()
            .map(|index| index == 0)
            .unwrap_or(false);
        let is_at_top_from_output_list = self
            .output_state
            .selected()
            .map(|index| index == 0)
            .unwrap_or(false);

        if is_in_output_list {
            if is_at_top_from_output_list {
                self.output_state.select(None);
                self.input_state.select_last();
            } else {
                self.output_state.select_previous();
            }
        } else if is_in_input_list {
            if is_at_top_from_input_list {
                self.output_state.select_last();
                self.input_state.select(None);
            } else {
                self.input_state.select_previous();
            }
        } else {
            self.output_state.select_first();
        }
    }

    fn selected(&self) -> Option<(String, bool)> {
        if let Some(index) = self.output_state.selected() {
            return Some((self.output_devices[index].clone(), true));
        }

        if let Some(index) = self.input_state.selected() {
            return Some((self.input_devices[index].clone(), false));
        }

        return None;
    }
}

impl Model for DeviceChooser {
    fn draw(&mut self, frame: &mut ratatui::Frame, _: WindowSize) {
        let column = Layout::horizontal([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(frame.area())[1];

        let columns = Layout::vertical([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(column);

        let title_layout = columns[0];
        let layout = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(columns[1]);

        let output_devices_list = List::new(self.output_devices.clone())
            .block(Block::bordered().title("Output devices"))
            .highlight_symbol(">> ")
            .highlight_style(Style::new().fg(Color::Blue));

        let input_devices_list = List::new(self.input_devices.clone())
            .block(Block::bordered().title("Input devices"))
            .highlight_symbol(">> ")
            .highlight_style(Style::new().fg(Color::Blue));

        let title = Line::styled(
            "Select an audio device for the audio source",
            (Modifier::BOLD, Modifier::UNDERLINED),
        )
        .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(title, title_layout);
        frame.render_stateful_widget(output_devices_list, layout[0], &mut self.output_state);
        frame.render_stateful_widget(input_devices_list, layout[1], &mut self.input_state);
    }

    fn handle_event(&mut self, event: Event) -> Action {
        if let Event::Key(KeyEvent { code, .. }) = event {
            match code {
                KeyCode::Char('q') => return Action::Quit,
                KeyCode::Char('j') => self.select_next(),
                KeyCode::Char('k') => self.select_previous(),
                KeyCode::Enter => {
                    if let Some((device_name, is_output_device)) = self.selected() {
                        return Action::StartVisualizer {
                            device_name,
                            is_output_device,
                        };
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
