use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::{WavSpec, WavWriter, WavReader, SampleFormat};
use std::path::Iter;
use std::sync::{Arc, Mutex};
use crate::strided_chunks::StridedChunks;

use crate::audio_controls::{RecordingControl, PlaybackControl};

pub trait Audio {
    fn sample_rate(&self) -> u32;
}

pub trait IterableAudio: Audio {
    fn sliding_windows<'a>(&'a self, window_size: usize, step_size: usize) -> impl Iterator<Item = Self>
    where
        Self: Sized;
}

pub trait MonoAudioSource: Audio {
    fn mono_samples(&self) -> &[f32];
}

#[derive(Debug, Clone)]
pub struct MonoAudio {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}

impl MonoAudio {
    pub fn new(samples: Vec<f32>, sample_rate: u32) -> Self {
        MonoAudio { samples, sample_rate }
    }
}
impl Audio for MonoAudio {
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}
impl MonoAudioSource for MonoAudio {
    fn mono_samples(&self) -> &[f32] {
        &self.samples
    }
}

impl IterableAudio for MonoAudio {
    fn sliding_windows(&self, window_size: usize, step_size: usize) -> impl Iterator<Item = MonoAudio> {
        StridedChunks::new(self.samples.clone(), window_size, step_size)
            .map(move |chunk| MonoAudio::new(chunk.to_vec(), self.sample_rate))
    }
}


/// FIXME below this line

/// Load audio samples and sample rate from a file path
pub fn load_audio_samples_and_rate(path: &str) -> Result<(Vec<f32>, u32), Box<dyn std::error::Error>> {
    let mut reader = WavReader::open(path)?;
    let spec = reader.spec();
    let samples: Vec<f32> = if spec.sample_format == SampleFormat::Float {
        reader.samples::<f32>().filter_map(Result::ok).collect()
    } else {
        reader.samples::<i16>().filter_map(Result::ok).map(|s| s as f32 / i16::MAX as f32).collect()
    };
    Ok((samples, spec.sample_rate))
}

pub fn record_audio_with_control_and_buffer(wav_path: &str, control: RecordingControl, live_buffer: Arc<Mutex<Vec<f32>>>) {
    record_audio_with_control_and_buffer_optional_save(Some(wav_path), control, live_buffer);
}

pub fn record_audio_with_control_and_buffer_optional_save(wav_path: Option<&str>, control: RecordingControl, live_buffer: Arc<Mutex<Vec<f32>>>) {
    let host = cpal::default_host();
    let device = host.default_input_device().expect("Failed to find input device");
    let supported_config = device.default_input_config().expect("Failed to get default input config");
    let stream_config: cpal::StreamConfig = supported_config.clone().into();

    println!("Recording from: {}", device.name().unwrap());
    let (tx, rx) = std::sync::mpsc::channel();
    let tx_callback = tx.clone();

    let stream = device.build_input_stream(
        &stream_config,
        {
            let live_buffer_clone = live_buffer.clone();
            let supported_sample_rate = supported_config.sample_rate().0;
            move |data: &[f32], _| {
                let _ = tx_callback.send(data.to_vec());
                // Push to live buffer for GUI
                if let Ok(mut buf) = live_buffer_clone.lock() {
                    buf.extend_from_slice(data);
                    // Limit buffer size for performance (e.g., last 10 seconds)
                    let max_samples = (supported_sample_rate * 10) as usize;
                    if buf.len() > max_samples {
                        let excess = buf.len() - max_samples;
                        buf.drain(0..excess);
                    }
                }
            }
        },
        move |err| eprintln!("Stream error: {:?}", err),
        None,
    ).expect("Failed to build input stream");
    stream.play().expect("Failed to start stream");

    println!("Recording... Press Stop to finish.");
    while !control.should_stop() {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    drop(stream);
    drop(tx);

    let mut samples = Vec::new();
    while let Ok(data) = rx.recv() {
        samples.extend(data);
    }

    let spec = WavSpec {
        channels: supported_config.channels(),
        sample_rate: supported_config.sample_rate().0,
        bits_per_sample: 32,
        sample_format: SampleFormat::Float,
    };
    
    // Only save to file if path is provided
    if let Some(path) = wav_path {
        let mut writer = WavWriter::create(path, spec).expect("WAV writer");
        for &s in &samples {
            writer.write_sample(s).unwrap();
        }
        writer.finalize().unwrap();
        println!("Saved to {path}");
    }
}

pub fn play_audio_with_control_and_notify_cleaned(path: &str, control: PlaybackControl, done_tx: std::sync::mpsc::Sender<()>, use_cleaning: bool) {
    let mut reader = WavReader::open(path).expect("WAV open");
    let spec = reader.spec();
    let mut samples: Vec<f32> = if spec.sample_format == SampleFormat::Float {
        reader.samples::<f32>().map(|s| s.unwrap()).collect()
    } else {
        reader.samples::<i16>().map(|s| s.unwrap() as f32 / i16::MAX as f32).collect()
    };
    if use_cleaning {
        let noise_spectrum = crate::signal_cleaning::estimate_noise_spectrum(&samples, spec.sample_rate as f32);
        samples = crate::signal_cleaning::clean_signal_for_pitch(&samples, spec.sample_rate as f32, Some(&noise_spectrum), None);
    }
    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device");
    let out_config = cpal::StreamConfig {
        channels: spec.channels,
        sample_rate: cpal::SampleRate(spec.sample_rate),
        buffer_size: cpal::BufferSize::Default,
    };
    let idx = control.sample_index.clone();
    let idx_clone = idx.clone();
    let total = samples.len();
    let control_clone = control.clone();
    let stream = device.build_output_stream(
        &out_config,
        move |out: &mut [f32], _| {
            for sample in out.iter_mut() {
                if control_clone.should_stop() {
                    *sample = 0.0;
                    continue;
                }
                let i = idx_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                *sample = if i < total { samples[i] } else { 0.0 };
            }
        },
        move |err| eprintln!("Stream error: {:?}", err),
        None,
    ).unwrap();
    stream.play().unwrap();
    println!("Playing...");
    while idx.load(std::sync::atomic::Ordering::SeqCst) < total && !control.should_stop() {
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    drop(stream);
    let _ = done_tx.send(());
}
