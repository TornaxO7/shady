use std::sync::{Arc, Mutex};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleFormat, StreamError, SupportedStreamConfig,
};
use ringbuffer::{AllocRingBuffer, RingBuffer};
use tracing::debug;

use crate::{Data, REQUIRED_SAMPLE_RATE, SAMPLE_RATE};

pub struct SystemAudio {
    data_snapshot: Arc<Mutex<AllocRingBuffer<f32>>>,
    _stream: cpal::Stream,
}

impl SystemAudio {
    pub fn boxed<E>(
        device: Option<&cpal::Device>,
        stream_config: Option<&SupportedStreamConfig>,
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
            let supported_stream_config = stream_config.unwrap_or(&default_output_config);

            supported_stream_config.config()
        };

        let data_snapshot: Arc<Mutex<AllocRingBuffer<f32>>> =
            Arc::new(Mutex::new(AllocRingBuffer::new(SAMPLE_RATE)));

        let stream = device
            .build_input_stream(
                &stream_config,
                {
                    let buffer = data_snapshot.clone();
                    move |input_samples: &[f32], _: &cpal::InputCallbackInfo| {
                        let mut buf = buffer.lock().unwrap();
                        buf.extend(input_samples.iter().cloned());
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
        let audio = self.data_snapshot.lock().unwrap();

        for i in 0..SAMPLE_RATE {
            buf[i] = *audio.get(i).unwrap_or(&0.);
        }
    }
}

fn default_output_config(device: &cpal::Device) -> SupportedStreamConfig {
    let mut matching_configs: Vec<_> = device
        .supported_output_configs()
        .expect("Get supported output configs of device")
        .filter(|entry| {
            entry.channels() == 1
                && entry.sample_format() == SampleFormat::F32
                && entry.min_sample_rate() <= REQUIRED_SAMPLE_RATE
        })
        .collect();

    matching_configs.sort_by(|a, b| a.cmp_default_heuristics(b));

    match matching_configs.into_iter().next() {
        Some(config) => {
            debug!("Found matching output config: {:?}", config);
            config.with_max_sample_rate()
        }
        None => {
            debug!("Didn't find matching output config. Fallback to default_output_config.");
            device
                .default_output_config()
                .expect("Get default output config for device")
        }
    }
}
