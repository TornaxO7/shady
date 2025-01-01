use shady_audio::{config::ShadyAudioConfig, fetcher::SystemAudioFetcher, ShadyAudio};
use wgpu::Device;

use super::Resource;

const AUDIO_BUFFER_SIZE: usize = 10;

pub struct Audio {
    shady_audio: ShadyAudio,

    audio_buffer: Box<[f32; AUDIO_BUFFER_SIZE]>,

    buffer: wgpu::Buffer,
    binding: u32,
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

    fn new(device: &Device, binding: u32) -> Self {
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
            binding,
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

    fn binding(&self) -> u32 {
        self.binding
    }

    fn update_buffer(&self, queue: &mut wgpu::Queue) {
        let data = &self.audio_buffer;
        queue.write_buffer(self.buffer(), 0, bytemuck::cast_slice(data.as_slice()));
    }
}
