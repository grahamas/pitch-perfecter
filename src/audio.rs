use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::{WavSpec, WavWriter, WavReader, SampleFormat};
use std::sync::{Arc, Mutex};

use crate::{RecordingControl, PlaybackControl};

pub fn play_audio_with_control_and_notify(path: &str, control: PlaybackControl, done_tx: std::sync::mpsc::Sender<()>) {
    let mut reader = WavReader::open(path).expect("WAV open");
    let spec = reader.spec();
    let samples: Vec<f32> = if spec.sample_format == SampleFormat::Float {
        reader.samples::<f32>().map(|s| s.unwrap()).collect()
    } else {
        reader.samples::<i16>().map(|s| s.unwrap() as f32 / i16::MAX as f32).collect()
    };
    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device");
    let out_config = cpal::StreamConfig {
        channels: spec.channels,
        sample_rate: cpal::SampleRate(spec.sample_rate),
        buffer_size: cpal::BufferSize::Default,
    };
    let idx = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
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
    // Wait until all samples are played or stop is requested
    while idx.load(std::sync::atomic::Ordering::SeqCst) < total && !control.should_stop() {
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    drop(stream);
    let _ = done_tx.send(());
}

pub fn record_audio_with_control_and_buffer(wav_path: &str, control: RecordingControl, live_buffer: Arc<Mutex<Vec<f32>>>) {
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
    let mut writer = WavWriter::create(wav_path, spec).expect("WAV writer");
    for &s in &samples {
        writer.write_sample(s).unwrap();
    }
    writer.finalize().unwrap();
    println!("Saved to {wav_path}");
}
