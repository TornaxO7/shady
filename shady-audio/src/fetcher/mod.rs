use std::sync::{Arc, Mutex};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleFormat, SampleRate, StreamError, SupportedStreamConfigRange,
};

use crate::{fft, Data, DEFAULT_SAMPLE_RATE};

const BUFFER_SIZE: usize = fft::FFT_INPUT_SIZE;

pub struct SystemAudio {
    data_buffer: Arc<Mutex<Vec<f32>>>,
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

        let data_buffer = Arc::new(Mutex::new(Vec::with_capacity(BUFFER_SIZE)));

        let stream = device
            .build_input_stream(
                &stream_config,
                {
                    let buffer = data_buffer.clone();
                    debug_assert_eq!(
                        stream_config.channels, 1,
                        "We are currently only supporting 1 channel"
                    );

                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        let mut buf = buffer.lock().unwrap();

                        let buf_len = buf.len();
                        // don't let the vec exceed the capacity
                        let new_len = std::cmp::min(buf_len + data.len(), BUFFER_SIZE - 1);
                        buf.resize(new_len, 0.);
                        // prepare the space for the new data
                        buf.copy_within(..buf_len, new_len - buf_len);
                        // put the new data to the beginning of the vec
                        buf[..data.len()].copy_from_slice(data);

                        debug_assert_eq!(
                            buf.capacity(),
                            BUFFER_SIZE,
                            "The buffer should be fixed sized!"
                        );
                    }
                },
                error_callback,
                None,
            )
            .expect("Start audio listening");

        stream.play().expect("Start listening to audio");

        let fetcher = Box::new(Self {
            _stream: stream,
            data_buffer,
        });

        fetcher
    }
}

impl Data for SystemAudio {
    fn fetch_snapshot(&mut self, buf: &mut [f32]) {
        let mut audio = self.data_buffer.lock().unwrap();

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
