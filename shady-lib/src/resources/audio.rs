use std::{
    fmt,
    num::{NonZeroU32, NonZeroUsize},
    ops::Range,
};

use shady_audio::{
    config::ShadyAudioConfig,
    fetcher::{Fetcher, SystemAudioFetcher},
    ShadyAudio,
};
use wgpu::Device;

use crate::template::TemplateGenerator;

use super::Resource;

const DEFAULT_AMOUNT_BARS: usize = 60;
const DESCRIPTION: &str = "\
// It contains the 'presence' of a frequency. The lower the index the lower is its frequency and the other way round.
// So for example, if you are interested in the bass, choose the lower indices.";

pub struct Audio {
    shady_audio: ShadyAudio,

    bar_values: Box<[f32]>,

    buffer: wgpu::Buffer,
}

impl Audio {
    pub fn fetch_audio(&mut self) {
        let bars = self.shady_audio.get_bars();

        self.bar_values.copy_from_slice(bars);
    }

    pub fn set_bars(&mut self, amount_bars: NonZeroUsize) {
        self.shady_audio.set_bars(amount_bars);
        self.bar_values = vec![0.; usize::from(amount_bars)].into_boxed_slice();
    }

    pub fn set_frequency_range(&mut self, freq_range: Range<NonZeroU32>) -> Result<(), ()> {
        self.shady_audio.set_freq_range(freq_range)
    }

    pub fn set_fetcher(&mut self, fecther: Box<dyn Fetcher>) {
        self.shady_audio.set_fetcher(fetcher);
    }
}

impl Resource for Audio {
    type BufferDataType = [f32; DEFAULT_AMOUNT_BARS];

    fn new(device: &Device) -> Self {
        let buffer = Self::create_storage_buffer(device);

        let shady_audio = ShadyAudio::new(
            SystemAudioFetcher::default(|err| panic!("{}", err)).unwrap(),
            ShadyAudioConfig {
                amount_bars: NonZeroUsize::new(DEFAULT_AMOUNT_BARS).unwrap(),
                ..Default::default()
            },
        )
        .unwrap();

        let audio_buffer = Box::new([0.; DEFAULT_AMOUNT_BARS]);

        Self {
            shady_audio,
            bar_values: audio_buffer,
            buffer,
        }
    }

    fn buffer_label() -> &'static str {
        "Shady iAudio buffer"
    }

    fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    fn buffer_type() -> wgpu::BufferBindingType {
        wgpu::BufferBindingType::Storage { read_only: true }
    }

    fn binding() -> u32 {
        super::BindingValue::Audio as u32
    }

    fn update_buffer(&self, queue: &mut wgpu::Queue) {
        let bars = &self.bar_values;
        queue.write_buffer(self.buffer(), 0, bytemuck::cast_slice(bars));
    }
}

impl TemplateGenerator for Audio {
    fn write_wgsl_template(
        writer: &mut dyn std::fmt::Write,
        bind_group_index: u32,
    ) -> Result<(), fmt::Error> {
        writer.write_fmt(format_args!(
            "
{}
@group({}) @binding({})
var<storage, read> iAudio: array<f32>;
",
            DESCRIPTION,
            bind_group_index,
            Self::binding(),
        ))
    }

    fn write_glsl_template(writer: &mut dyn fmt::Write) -> Result<(), fmt::Error> {
        writer.write_fmt(format_args!(
            "
{}
layout(binding = {}) readonly buffer iAudio {{
    float[] freqs;
}};
",
            DESCRIPTION,
            Self::binding(),
        ))
    }
}
