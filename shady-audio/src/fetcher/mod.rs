use std::sync::{Arc, Mutex};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleFormat, SampleRate, StreamError, SupportedStreamConfigRange,
};

use crate::{fft, Data, DEFAULT_SAMPLE_RATE};

pub struct SystemAudio {
    data_snapshot: Arc<Mutex<Vec<f32>>>,
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

        let data_snapshot = Arc::new(Mutex::new(Vec::with_capacity(fft::FFT_INPUT_SIZE)));

        let stream = device
            .build_input_stream(
                &stream_config,
                {
                    let buffer = data_snapshot.clone();
                    debug_assert_eq!(stream_config.channels, 1);

                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        let mut buf = buffer.lock().unwrap();

                        let buf_len = buf.len();
                        buf.resize(buf_len + data.len(), 0.);

                        buf[buf_len..].copy_from_slice(data);
                    }
                },
                error_callback,
                None,
            )
            .expect("Start audio listening");

        stream.play().expect("Start listening to audio");

        let fetcher = Box::new(Self {
            _stream: stream,
            data_snapshot,
        });

        fetcher
    }
}

impl Data for SystemAudio {
    fn fetch_snapshot(&mut self, buf: &mut [f32]) {
        let mut audio = self.data_snapshot.lock().unwrap();

        // adjust buf
        let audio_len = audio.len();
        let buf_len = buf.len();
        let mid = std::cmp::min(audio_len, buf_len);

        buf.copy_within(..(buf_len - mid), mid);
        buf[..mid].copy_from_slice(&audio);

        audio.clear();
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
