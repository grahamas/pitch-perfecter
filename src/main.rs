use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::{WavSpec, WavWriter, WavReader, SampleFormat};
use std::sync::mpsc;
use std::thread;

fn main() {
    // Set up the audio host and input device
    let host = cpal::default_host();
    let device = host.default_input_device().expect("Failed to find input device");
    // Get the supported config and clone for stream and WAV spec
    let supported_config = device.default_input_config().expect("Failed to get default input config");
    let stream_config: cpal::StreamConfig = supported_config.clone().into();

    println!("Recording from: {}", device.name().unwrap());
    // Create a channel to receive audio data
    let (tx, rx) = mpsc::channel();
    // Clone the sender for the audio callback, keep original to close channel later
    let tx_callback = tx.clone();

    // Build the input stream
    let stream = device.build_input_stream(
        &stream_config,
        move |data: &[f32], _| { let _ = tx_callback.send(data.to_vec()); },
        move |err| {
            eprintln!("Stream error: {:?}", err);
        },
        None,
    ).expect("Failed to build input stream");

    // Start the stream
    stream.play().expect("Failed to start stream");

    println!("Recording... Press Enter to stop.");
    // Wait for user input to stop recording
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).unwrap();

    // Stop the stream by dropping it (which drops the callback sender)
    drop(stream);
    drop(tx);

    // Collect all recorded samples from the channel
    let mut samples = Vec::new();
    while let Ok(data) = rx.recv() {
        samples.extend(data);
    }

    // Write samples to a WAV file
    let spec = WavSpec {
        channels: supported_config.channels(),
        sample_rate: supported_config.sample_rate().0,
        bits_per_sample: 32,
        sample_format: SampleFormat::Float,
    };
    let mut writer = WavWriter::create("recorded_audio.wav", spec).expect("WAV writer");
    for &s in &samples {
        writer.write_sample(s).unwrap();
    }
    writer.finalize().unwrap();

    println!("Saved to recorded_audio.wav; playing back...");
    // Play the recorded audio after recording
    play_audio("recorded_audio.wav");
}

fn play_audio(path: &str) {
    // Read WAV data from file
    let mut reader = WavReader::open(path).expect("WAV open");
    let spec = reader.spec();
    let samples: Vec<f32> = if spec.sample_format == SampleFormat::Float {
        reader.samples::<f32>().map(|s| s.unwrap()).collect()
    } else {
        reader.samples::<i16>().map(|s| s.unwrap() as f32 / i16::MAX as f32).collect()
    };
    // Set up the audio host and output device
    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device");
    // Configure output stream to match WAV file's sample rate and channels
    let out_config = cpal::StreamConfig {
        channels: spec.channels,
        sample_rate: cpal::SampleRate(spec.sample_rate),
        buffer_size: cpal::BufferSize::Default,
    };

    let mut idx = 0;
    let total = samples.len();
    // Build the output stream
    let stream = device.build_output_stream(
        &out_config,
        move |out: &mut [f32], _| {
            for sample in out.iter_mut() {
                *sample = if idx < total { samples[idx] } else { 0.0 };
                idx += 1;
            }
        },
        move |err| {
            eprintln!("Stream error: {:?}", err);
        },
        None,
    ).unwrap();
    // Start the stream
    stream.play().unwrap();
    println!("Playing... Press Ctrl+C to stop.");

    // Keep the main thread alive while playing
    loop {
        thread::park();
    }
}