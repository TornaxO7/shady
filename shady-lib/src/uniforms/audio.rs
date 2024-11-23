use std::sync::{Arc, Mutex};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleRate, Stream,
};
use realfft::{num_complex::Complex32, RealFftPlanner};

const AUDIO_BUFFER_SIZE: usize = 3;

pub struct AudioData {
    i_audio: Arc<Mutex<[f32; AUDIO_BUFFER_SIZE]>>,
    stream: Stream,
    _fft: Arc<Mutex<FftWrapper>>,
}

impl AudioData {
    pub fn new() -> Self {
        let fft = Arc::new(Mutex::new(FftWrapper::new()));
        let i_audio = Arc::new(Mutex::new([0f32; AUDIO_BUFFER_SIZE]));

        let stream = {
            let host = cpal::default_host();

            let device = host
                .default_output_device()
                .expect("Get default output device");

            let config = device
                .default_input_config()
                .expect("Shady: Get default input config of device")
                .config();

            device
                .build_input_stream(
                    &config,
                    {
                        let sample_rate = config.sample_rate;
                        let fft_clone = fft.clone();
                        let i_audio_clone = i_audio.clone();

                        move |data: &[f32], _: &cpal::InputCallbackInfo| {
                            let data_len = data.len();
                            let mut fft = fft_clone.lock().unwrap();

                            let fourier_output = fft.process(data);

                            let (bass, mid, treble) =
                                split_audio(fourier_output, data_len, sample_rate);

                            let mut i_audio = i_audio_clone.lock().unwrap();
                            i_audio[0] = average(&bass);
                            i_audio[1] = average(&mid);
                            i_audio[2] = average(&treble);
                        }
                    },
                    move |err| {
                        eprintln!("Audio skill issue: {}", err);
                    },
                    None,
                )
                .expect("Create stream")
        };

        stream.play().unwrap();

        Self {
            i_audio,
            stream,
            _fft: fft,
        }
    }

    pub fn data(&self) -> [f32; AUDIO_BUFFER_SIZE] {
        let mut buffer = [0f32; AUDIO_BUFFER_SIZE];
        buffer.copy_from_slice(&*self.i_audio.lock().unwrap());
        buffer
    }

    pub fn cleanup(&mut self) {
        self.stream.pause().expect("Cleanup audio stream");
    }
}

struct FftWrapper {
    planner: RealFftPlanner<f32>,
    output: Vec<Complex32>,
}

impl FftWrapper {
    pub fn new() -> Self {
        let planner = RealFftPlanner::new();

        Self {
            planner,
            output: Vec::new(),
        }
    }

    pub fn process(&mut self, data: &[f32]) -> &[Complex32] {
        let size = data.len();
        let fft = self.planner.plan_fft_forward(size);

        let mut input = data.to_vec();
        self.output = fft.make_output_vec();

        fft.process(&mut input, &mut self.output)
            .expect("Calculate fourier transformation");

        &self.output
    }
}

fn split_audio(
    fourier_output: &[Complex32],
    data_len: usize,
    sample_rate: SampleRate,
) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    let mut bass = Vec::new();
    let mut mid = Vec::new();
    let mut treble = Vec::new();

    for (i, freq_bin) in fourier_output.iter().enumerate() {
        let frequency = i as f32 * sample_rate.0 as f32 / data_len as f32;
        let magnitude = freq_bin.norm();

        if frequency < 250. {
            bass.push(magnitude);
        } else if frequency < 4_000. {
            mid.push(magnitude);
        } else if frequency < 20_000. {
            treble.push(magnitude);
        }
    }

    (bass, mid, treble)
}

fn average(magnitudes: &[f32]) -> f32 {
    magnitudes.iter().sum::<f32>() / magnitudes.len() as f32
}
