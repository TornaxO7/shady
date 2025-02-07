use std::sync::{Arc, Mutex};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleFormat, SampleRate, StreamError, SupportedStreamConfigRange,
};

use super::Fetcher;

struct SampleBuffer {
    buffer: Box<[f32]>,
    length: usize,
    capacity: usize,
}

impl SampleBuffer {
    pub fn new(sample_rate: SampleRate) -> Self {
        let capacity = (sample_rate.0 * 10) as usize;
        let buffer = vec![0.; capacity].into_boxed_slice();

        Self {
            buffer,
            capacity,
            length: 0,
        }
    }

    pub fn push_before(&mut self, data: &[f32]) {
        let new_len = std::cmp::min(self.capacity, self.length + data.len());

        // move the current values to the right
        self.buffer
            .copy_within(..self.length, new_len - self.length);
        // copy the new data to the beginning
        self.buffer[..data.len()].copy_from_slice(data);

        self.length += data.len();
    }

    pub fn clear(&mut self) {
        self.length = 0;
    }
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
    ///
    /// Currently only devices and configs are supported which:
    ///     - are able to have the sample rate set by [DEFAULT_SAMPLE_RATE].
    ///     - *and* have exactlly *one* channel
    ///
    /// It's ***strongly recommended*** to use [SystemAudio::default] instead to reduce the headache.
    pub fn new<E>(
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

            assert!(
                supported_stream_config.channels() == 1,
                "ShadyAudio currently supports only configs with one channel. Your config has set it to {} channels",
                supported_stream_config.channels()
            );

            supported_stream_config.with_max_sample_rate().config()
        };

        let sample_rate = stream_config.sample_rate;

        let sample_buffer = {
            let buffer = SampleBuffer::new(sample_rate);
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

        Box::new(Self {
            _stream: stream,
            sample_buffer,
            sample_rate,
        })
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
    pub fn default<E>(error_callback: E) -> Box<Self>
    where
        E: FnMut(StreamError) + Send + 'static,
    {
        Self::new(None, None, error_callback)
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

fn default_output_config(device: &cpal::Device) -> SupportedStreamConfigRange {
    let mut matching_configs: Vec<_> = device
        .supported_output_configs()
        .expect("Get supported output configs of device")
        .filter(|entry| entry.channels() == 1 && entry.sample_format() == SampleFormat::F32)
        .collect();

    matching_configs.sort_by(|a, b| a.cmp_default_heuristics(b));

    matching_configs
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("Couldn't find suitable config"))
}
