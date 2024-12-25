use std::time::Duration;

use crossterm::{
    event::{self, Event},
    terminal::WindowSize,
};
use ratatui::{
    style::Style,
    widgets::{Bar, BarChart, BarGroup},
    Frame,
};
use shady_audio::ShadyAudio;

fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let mut audio = ShadyAudio::default_with_callback(|err| panic!("{}", err));

    loop {
        let window_size = crossterm::terminal::window_size()?;

        // let Some(spline) = audio.spline(0f32..10f32, splines::Interpolation::Cosine) else {
        //     continue;
        // };

        // let mut buffer = Vec::new();

        // for i in 0..10 {
        //     match spline.clamped_sample(i as f32) {
        //         Some(value) => buffer.push(value),
        //         None => buffer.push(f32::NAN),
        //     };
        // }
        // println!("{:?}", buffer);

        terminal
            .draw(|frame| draw(frame, &mut audio, window_size))
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

fn draw(frame: &mut Frame, audio: &mut ShadyAudio, window_size: WindowSize) {
    const MAX_HEIGHT: u64 = 100;

    let bar_group = {
        let spline = audio.next_spline();

        let mut bars = Vec::with_capacity(window_size.columns.into());
        for column in 0..window_size.columns {
            let value = spline
                .clamped_sample(column as f32 / window_size.columns as f32)
                .unwrap_or(0.0);

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
        .bar_width(1)
        .max(MAX_HEIGHT);

    frame.render_widget(&bar_chart, frame.area());
}
