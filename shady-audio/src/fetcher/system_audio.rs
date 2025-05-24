use std::sync::{Arc, Mutex};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleRate, StreamError, SupportedStreamConfigRange,
};
use tracing::{debug, instrument};

use crate::DEFAULT_SAMPLE_RATE;

use super::Fetcher;

struct SampleBuffer {
    buffer: Box<[f32]>,
    length: usize,
    capacity: usize,
    channels: u16,
}

impl SampleBuffer {
    pub fn new(sample_rate: SampleRate, channels: u16) -> Self {
        let capacity = sample_rate.0 as usize;
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

        self.length = new_len;
    }
}

/// Errors which can occur while creating [crate::fetcher::SystemAudioFetcher].
#[derive(thiserror::Error, Debug, Clone, Copy)]
pub enum SystemAudioError {
    /// No default audio device could be found to fetch from.
    #[error("Couldn't retrieve default output dev")]
    NoDefaultDevice,

    /// No default configuration could be found of the default output device.
    #[error("Couldn't retrieve any config of the output stream of the default device.")]
    NoAvailableOutputConfigs,
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
    #[instrument(name = "SystemAudio::new", skip_all)]
    pub fn new<E>(
        device: &cpal::Device,
        stream_config_range: &SupportedStreamConfigRange,
        error_callback: E,
    ) -> Result<Box<Self>, SystemAudioError>
    where
        E: FnMut(StreamError) + Send + 'static,
    {
        let stream_config = {
            let supported_stream_config = stream_config_range
                .try_with_sample_rate(DEFAULT_SAMPLE_RATE)
                .unwrap_or(stream_config_range.with_max_sample_rate());
            supported_stream_config.config()
        };
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
    pub fn default<E>(error_callback: E) -> Result<Box<Self>, SystemAudioError>
    where
        E: FnMut(StreamError) + Send + 'static,
    {
        let Some(default_device) = cpal::default_host().default_output_device() else {
            return Err(SystemAudioError::NoDefaultDevice);
        };

        let default_stream_config = default_output_config(&default_device)?;

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
    fn fetch_samples(&mut self, buf: &mut [f32]) {
        let buf_len = buf.len();
        let mut sample_buffer = self.sample_buffer.lock().unwrap();

        let amount_samples = buf_len.min(sample_buffer.length);
        let new_sample_buffer_len = sample_buffer.length - amount_samples;

        buf.copy_within(..buf_len - amount_samples, amount_samples);
        buf[..amount_samples]
            .copy_from_slice(&sample_buffer.buffer[new_sample_buffer_len..sample_buffer.length]);

        sample_buffer.length = new_sample_buffer_len;
    }

    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }
}

#[instrument(skip_all)]
fn default_output_config(
    device: &cpal::Device,
) -> Result<SupportedStreamConfigRange, SystemAudioError> {
    let mut matching_configs: Vec<_> = device
        .supported_output_configs()
        .expect(concat![
            "Eh... somehow `shady-audio` couldn't get any supported output configs of your audio device.\n",
            "Could it be that you are running \"pure\" pulseaudio?\n",
            "Only ALSA and JACK are supported for audio processing :("
        ])
        .collect();

    matching_configs.sort_by(|a, b| a.cmp_default_heuristics(b));
    matching_configs
        .into_iter()
        .next()
        .ok_or(SystemAudioError::NoAvailableOutputConfigs)
}
