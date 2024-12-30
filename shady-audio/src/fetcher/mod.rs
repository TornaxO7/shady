use std::sync::{Arc, Mutex};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleFormat, SampleRate, StreamError, SupportedStreamConfigRange,
};
use ringbuffer::{AllocRingBuffer, RingBuffer};

use crate::{Data, DEFAULT_SAMPLE_RATE};

pub struct SystemAudio {
    data_snapshot: Arc<Mutex<AllocRingBuffer<f32>>>,
    _stream: cpal::Stream,
}

impl SystemAudio {
    pub fn boxed<E>(
        device: Option<&cpal::Device>,
        stream_config_range: Option<&SupportedStreamConfigRange>,
        error_callback: E,
    ) -> Box<Self>
    where
        E: FnMut(StreamError) + Send + 'static,
    {
        let default_device = cpal::default_host()
            .default_output_device()
            .expect("Default output device exists");

        let device = device.unwrap_or(&default_device);

        let stream_config = {
            let default_output_config = default_output_config(device);
            let supported_stream_config = stream_config_range.unwrap_or(&default_output_config);

            supported_stream_config
                .try_with_sample_rate(SampleRate(DEFAULT_SAMPLE_RATE as u32))
                .unwrap_or_else(|| todo!("We currently support only stream configs which are able to provide a sample rate of 44.100Hz."))
                .config()
        };

        let data_snapshot: Arc<Mutex<AllocRingBuffer<f32>>> =
            Arc::new(Mutex::new(AllocRingBuffer::new(DEFAULT_SAMPLE_RATE)));

        let stream = device
            .build_input_stream(
                &stream_config,
                {
                    let buffer = data_snapshot.clone();
                    let amount_channels = stream_config.channels;

                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        let mut buf = buffer.lock().unwrap();

                        let chunks = data
                            .chunks_exact(amount_channels.into())
                            .map(|chunk| chunk.iter().sum::<f32>() / amount_channels as f32);

                        buf.extend(chunks);
                    }
                },
                error_callback,
                None,
            )
            .expect("Start audio listening");

        stream.play().expect("Start listening to audio");

        Box::new(Self {
            _stream: stream,
            data_snapshot,
        })
    }
}

impl Data for SystemAudio {
    fn fetch_snapshot(&mut self, buf: &mut [f32]) {
        debug_assert_eq!(buf.len(), DEFAULT_SAMPLE_RATE);

        let audio = self.data_snapshot.lock().unwrap();

        for (buf_val, &snap_val) in buf.iter_mut().zip(audio.iter()) {
            *buf_val = snap_val;
        }
    }
}

fn default_output_config(device: &cpal::Device) -> SupportedStreamConfigRange {
    let mut matching_configs: Vec<_> = device
        .supported_output_configs()
        .expect("Get supported output configs of device")
        .filter(|entry| {
            entry.channels() == 1
                && entry.sample_format() == SampleFormat::F32
                && entry.min_sample_rate() <= SampleRate(DEFAULT_SAMPLE_RATE as u32)
        })
        .collect();

    matching_configs.sort_by(|a, b| a.cmp_default_heuristics(b));

    matching_configs
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("Couldn't find suitable config"))
}
