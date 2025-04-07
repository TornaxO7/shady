use std::{fmt, num::NonZero, ops::Range};

use shady_audio::{BarProcessor, BarProcessorConfig, SampleProcessor};
use wgpu::Device;

use crate::{template::TemplateGenerator, ShadyDescriptor};

use super::Resource;

const DEFAULT_AMOUNT_BARS: usize = 60;
const DESCRIPTION: &str = "\
// It contains the 'presence' of a frequency. The lower the index the lower is its frequency and the other way round.
// So for example, if you are interested in the bass, choose the lower indices.";

pub struct Audio {
    bar_processor: BarProcessor,

    bar_values: Box<[f32]>,

    buffer: wgpu::Buffer,
}

impl Audio {
    pub fn fetch_audio(&mut self, sample_processor: &SampleProcessor) {
        let bars = self.bar_processor.process_bars(sample_processor);
        self.bar_values.copy_from_slice(bars);
    }

    pub fn set_bars(
        &mut self,
        device: &Device,
        sample_processor: &SampleProcessor,
        amount_bars: NonZero<u16>,
    ) {
        self.bar_processor = BarProcessor::new(
            sample_processor,
            BarProcessorConfig {
                amount_bars,
                ..self.bar_processor.config().clone()
            },
        );

        self.bar_values = vec![0.; usize::from(u16::from(amount_bars))].into_boxed_slice();

        self.buffer = Self::create_storage_buffer(
            device,
            (std::mem::size_of::<f32>() * usize::from(u16::from(amount_bars))) as u64,
        );
    }

    pub fn set_frequency_range(
        &mut self,
        sample_processor: &SampleProcessor,
        freq_range: Range<NonZero<u16>>,
    ) {
        self.bar_processor = BarProcessor::new(
            sample_processor,
            BarProcessorConfig {
                freq_range,
                ..self.bar_processor.config().clone()
            },
        );
    }
}

impl Resource for Audio {
    fn new(desc: &ShadyDescriptor) -> Self {
        let buffer = Self::create_storage_buffer(
            desc.device,
            std::mem::size_of::<[f32; DEFAULT_AMOUNT_BARS]>() as u64,
        );

        let bar_processor = BarProcessor::new(
            desc.sample_processor,
            BarProcessorConfig {
                amount_bars: NonZero::new(DEFAULT_AMOUNT_BARS as u16).unwrap(),
                ..Default::default()
            },
        );

        let audio_buffer = Box::new([0.; DEFAULT_AMOUNT_BARS]);

        Self {
            bar_processor,
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

    fn update_buffer(&self, queue: &wgpu::Queue) {
        queue.write_buffer(self.buffer(), 0, bytemuck::cast_slice(&self.bar_values));
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
