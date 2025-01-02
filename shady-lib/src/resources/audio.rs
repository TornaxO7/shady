use std::fmt;

use shady_audio::{config::ShadyAudioConfig, fetcher::SystemAudioFetcher, ShadyAudio};
use wgpu::Device;

use crate::template::TemplateGenerator;

use super::Resource;

const AUDIO_BUFFER_SIZE: usize = 20;

pub struct Audio {
    shady_audio: ShadyAudio,

    audio_buffer: Box<[f32; AUDIO_BUFFER_SIZE]>,

    buffer: wgpu::Buffer,
}

impl Audio {
    pub fn fetch_audio(&mut self) {
        let spline = self.shady_audio.get_spline();

        for i in 0..AUDIO_BUFFER_SIZE {
            let x = i as f32 / (AUDIO_BUFFER_SIZE + 1) as f32;
            self.audio_buffer[i] = spline.sample(x).unwrap_or(0.);
        }
    }
}

impl Resource for Audio {
    type BufferDataType = [f32; AUDIO_BUFFER_SIZE];

    fn new(device: &Device) -> Self {
        let buffer = Self::create_storage_buffer(device);

        let shady_audio = ShadyAudio::new(
            SystemAudioFetcher::default(|err| panic!("{}", err)),
            ShadyAudioConfig::default(),
        );

        let audio_buffer = Box::new([0.; AUDIO_BUFFER_SIZE]);

        Self {
            shady_audio,
            audio_buffer,
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
        let data = &self.audio_buffer;
        queue.write_buffer(self.buffer(), 0, bytemuck::cast_slice(data.as_slice()));
    }
}

impl TemplateGenerator for Audio {
    fn write_wgsl_template(
        writer: &mut dyn std::fmt::Write,
        bind_group_index: u32,
    ) -> Result<(), fmt::Error> {
        writer.write_fmt(format_args!(
            "
@group({}) @binding({})
var<storage, read> iAudio: array<f32, {}>;
",
            bind_group_index,
            Self::binding(),
            AUDIO_BUFFER_SIZE
        ))
    }

    fn write_glsl_template(writer: &mut dyn fmt::Write) -> Result<(), fmt::Error> {
        writer.write_fmt(format_args!(
            "
layout(binding = {}) buffer iAudio {{
    float freqs[{}];
}};
",
            Self::binding(),
            AUDIO_BUFFER_SIZE
        ))
    }
}
