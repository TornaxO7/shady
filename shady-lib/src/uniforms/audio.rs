use std::sync::{Arc, Mutex};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleRate, Stream,
};
use realfft::{num_complex::Complex32, RealFftPlanner};

use super::Uniform;

const AUDIO_BUFFER_SIZE: usize = 3;

pub struct Audio {
    data: Arc<Mutex<[f32; AUDIO_BUFFER_SIZE]>>,
    stream: Stream,
    _fft: Arc<Mutex<FftWrapper>>,
}

impl Uniform for Audio {
    type BufferDataType = [f32; AUDIO_BUFFER_SIZE];

    fn buffer_label() -> &'static str {
        "Shady iAudio buffer"
    }

    fn binding() -> u32 {
        2
    }

    fn update_buffer(&self, queue: &mut wgpu::Queue, device: &wgpu::Device) {
        let data = self.data.lock().unwrap();
        queue.write_buffer(&Self::buffer(device), 0, bytemuck::cast_slice(&*data));
    }

    fn cleanup(&mut self) {
        self.stream.pause().expect("Close audio stream");
    }
}

impl Audio {
    pub fn new() -> Self {
        let fft = Arc::new(Mutex::new(FftWrapper::new()));
        let data = Arc::new(Mutex::new([0f32; AUDIO_BUFFER_SIZE]));

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
                        let data_clone = data.clone();

                        move |data: &[f32], _: &cpal::InputCallbackInfo| {
                            let data_len = data.len();
                            let mut fft = fft_clone.lock().unwrap();

                            let fourier_output = fft.process(data);

                            let (bass, mid, treble) =
                                split_audio(fourier_output, data_len, sample_rate);

                            let mut uniform_data = data_clone.lock().unwrap();
                            uniform_data[0] = average(&bass);
                            uniform_data[1] = average(&mid);
                            uniform_data[2] = average(&treble);
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
            data,
            stream,
            _fft: fft,
        }
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
