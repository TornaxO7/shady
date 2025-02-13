use std::sync::{Arc, Mutex};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleFormat, SampleRate, StreamError, SupportedStreamConfigRange,
};
use tracing::{debug, instrument};

use super::Fetcher;

struct SampleBuffer {
    buffer: Box<[f32]>,
    length: usize,
    capacity: usize,
    channels: u16,
}

impl SampleBuffer {
    pub fn new(sample_rate: SampleRate, channels: u16) -> Self {
        let capacity = (sample_rate.0 * 10) as usize;
        let buffer = vec![0.; capacity].into_boxed_slice();

        Self {
            buffer,
            capacity,
            length: 0,
            channels,
        }
    }

    pub fn push_before(&mut self, data: &[f32]) {
        let data_len = data.len() / self.channels as usize;
        let new_len = std::cmp::min(self.capacity, self.length + data_len);

        // move the current values to the right
        self.buffer
            .copy_within(..self.length, new_len - self.length);

        for (i, values) in data.chunks_exact(self.channels as usize).enumerate() {
            self.buffer[i] = values.iter().sum::<f32>() / self.channels as f32;
        }

        self.length += data_len;
    }

    pub fn clear(&mut self) {
        self.length = 0;
    }
}

#[derive(thiserror::Error, Debug, Clone, Copy)]
pub enum SystemAudioError {
    #[error("Couldn't retrieve default output dev")]
    NoDefaultDevice,

    #[error("Expected sample format F32 but got {0} instead.")]
    InvalidSampleFormat(SampleFormat),

    #[error("Couldn't retrieve default config of the output stream of the default device.")]
    NoDefaultOutputStreamConfig,
}

/// Fetcher for the system audio.
///
/// It's recommended to use [SystemAudio::default] to create a new instance of this struct.
pub struct SystemAudio {
    sample_buffer: Arc<Mutex<SampleBuffer>>,
    sample_rate: SampleRate,

    _stream: cpal::Stream,
}

impl SystemAudio {
    /// This exposes the API of [cpal] which you can use to use your own [cpal::Device] and [cpal::SupportedStreamConfigRange]
    /// if you want.
    ///
    /// # Note
    /// It's required that the device supports `f32` as its sample format!
    #[instrument(name = "SystemAudio::new", skip_all)]
    pub fn new<E>(
        device: &cpal::Device,
        stream_config_range: &SupportedStreamConfigRange,
        error_callback: E,
    ) -> Result<Box<Self>, SystemAudioError>
    where
        E: FnMut(StreamError) + Send + 'static,
    {
        let sample_format = stream_config_range.sample_format();
        if sample_format != SampleFormat::F32 {
            return Err(SystemAudioError::InvalidSampleFormat(sample_format));
        }

        let stream_config = stream_config_range.with_max_sample_rate().config();
        let sample_rate = stream_config.sample_rate;

        debug!("Stream config: {:?}", stream_config);

        let sample_buffer = {
            let channels = stream_config.channels;
            let buffer = SampleBuffer::new(sample_rate, channels);
            Arc::new(Mutex::new(buffer))
        };

        let stream = {
            let stream = device
                .build_input_stream(
                    &stream_config,
                    {
                        let buffer = sample_buffer.clone();
                        move |data: &[f32], _: &cpal::InputCallbackInfo| {
                            let mut buf = buffer.lock().unwrap();
                            buf.push_before(data);
                        }
                    },
                    error_callback,
                    None,
                )
                .expect("Start audio listening");
            stream.play().expect("Start listening to audio");

            stream
        };

        Ok(Box::new(Self {
            _stream: stream,
            sample_buffer,
            sample_rate,
        }))
    }

    /// Equals `SystemAudio::new(None, None, error_fallback)`.
    ///
    /// Let's `ShadyAudio` pick up the device and config.
    ///
    /// This is the recommended function to create an instance of this struct.
    ///
    /// # Args
    /// - `error_callback` will be passed to the
    ///   `error_callback` of [`cpal::traits::DeviceTrait::build_input_stream`].
    ///
    /// # Example
    /// ```no_run
    /// use shady_audio::{ShadyAudio, config::ShadyAudioConfig, fetcher::SystemAudioFetcher};
    ///
    /// let shady = ShadyAudio::new(SystemAudioFetcher::default(|err| panic!("{}", err)), ShadyAudioConfig::default());
    /// ```
    pub fn default<E>(error_callback: E) -> Result<Box<Self>, SystemAudioError>
    where
        E: FnMut(StreamError) + Send + 'static,
    {
        let Some(default_device) = cpal::default_host().default_output_device() else {
            return Err(SystemAudioError::NoDefaultDevice);
        };

        let Some(default_stream_config) = default_output_config(&default_device) else {
            return Err(SystemAudioError::NoDefaultOutputStreamConfig);
        };

        Self::new(&default_device, &default_stream_config, error_callback)
    }
}

impl Drop for SystemAudio {
    /// Closes the audio stream before it gets dropped.
    ///
    /// **Panics** if it couldn't close the stream correctly.
    fn drop(&mut self) {
        self._stream.pause().expect("Stop stream");
    }
}

impl Fetcher for SystemAudio {
    fn fetch_samples(&mut self, buf: &mut Vec<f32>) {
        let mut sample_buffer = self.sample_buffer.lock().unwrap();
        buf.resize(sample_buffer.length, 0.);
        buf.copy_from_slice(&sample_buffer.buffer[..sample_buffer.length]);
        sample_buffer.clear();
    }

    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }
}

#[instrument(skip_all)]
fn default_output_config(device: &cpal::Device) -> Option<SupportedStreamConfigRange> {
    let mut matching_configs: Vec<_> = device
        .supported_output_configs()
        .expect("Get supported output configs of device")
        .filter(|entry| entry.channels() == 1 && entry.sample_format() == SampleFormat::F32)
        .collect();

    matching_configs.sort_by(|a, b| a.cmp_default_heuristics(b));
    matching_configs.into_iter().next()
}
