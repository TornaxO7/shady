use std::sync::{Arc, Mutex};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleFormat, SampleRate, StreamError, SupportedStreamConfigRange,
};

use super::{Fetcher, DEFAULT_SAMPLE_RATE};
use crate::fft;

const BUFFER_SIZE: usize = fft::FFT_INPUT_SIZE;

/// Fetcher for the system audio.
///
/// It's recommended to use [SystemAudio::default] to create a new instance of this struct.
pub struct SystemAudio {
    data_buffer: Arc<Mutex<Vec<f32>>>,
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

        Box::new(Self {
            _stream: stream,
            data_buffer,
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
    /// `error_callback` of [`cpal::traits::DeviceTrait::build_input_stream`].
    ///
    /// # Example
    /// ```no_run
    /// use shady_audio::{ShadyAudio, config::ShadyAudioConfig, fetcher::SystemAudioFetcher};
    ///
    /// fn main() {
    ///     let shady = ShadyAudio::new(SystemAudioFetcher::new(), ShadyAudioConfig::default());
    /// }
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
