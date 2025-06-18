// Pitch detection tests for the current pitch_track implementation
use pitch_perfecter::pitch_detection::pitch_track;
use plotters::prelude::*;
use hound::{WavWriter, WavSpec, SampleFormat};
use std::f32::consts::PI;

fn sine_wave(freq: f32, sample_rate: f32, len: usize) -> Vec<f32> {
    (0..len)
        .map(|i| (2.0 * PI * freq * i as f32 / sample_rate).sin())
        .collect()
}

#[test]
fn test_pitch_track_length() {
    let sample_rate = 16000.0;
    let freq = 440.0;
    let len = 4096;
    let window = 1024;
    let step = 256;
    let threshold = 0.15;
    let signal = sine_wave(freq, sample_rate, len);
    let pitches = pitch_track(&signal, sample_rate, window, step, threshold);
    // Basic sanity check: output length
    assert_eq!(pitches.len(), ((len - window) / step + 1));
}

#[test]
fn test_pitch_track_accuracy_across_range() {
    let sample_rate = 44100.0;
    let duration_sec = 1.0;
    let len = (sample_rate * duration_sec) as usize;
    let harmonics = 8;
    let window = 1024;
    let step = 256;
    let threshold = 0.15;
    let test_freqs: Vec<f32> = (0..10)
        .map(|i| 80.0f32 * (1000.0f32/80.0f32).powf(i as f32 / 9.0))
        .collect();
    let mut true_pitches = Vec::new();
    let mut detected_pitches = Vec::new();
    std::fs::create_dir_all("test_outputs").unwrap();
    println!("[DEBUG] Starting test_pitch_track_accuracy_across_range");
    for (idx, &freq) in test_freqs.iter().enumerate() {
        println!("[DEBUG] Generating signal for freq: {} Hz", freq);
        // Use a simple sine wave for each test frequency
        let signal = sine_wave(freq, sample_rate, len);
        let wav_path = format!("test_outputs/voice_{idx:02}_{:.0}Hz.wav", freq);
        let spec = WavSpec {
            channels: 1,
            sample_rate: sample_rate as u32,
            bits_per_sample: 32,
            sample_format: SampleFormat::Float,
        };
        let mut writer = WavWriter::create(&wav_path, spec).unwrap();
        for s in &signal { writer.write_sample(*s).unwrap(); }
        writer.finalize().unwrap();
        let pitches = pitch_track(&signal, sample_rate, window, step, threshold);
        let mut valid: Vec<f32> = pitches.into_iter().filter(|&p| p > 0.0).collect();
        valid.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = if valid.is_empty() { 0.0 } else { valid[valid.len()/2] };
        true_pitches.push(freq);
        detected_pitches.push(median);
        println!("[DEBUG] Median detected pitch for {:.1} Hz: {:.2} Hz", freq, median);
    }
    println!("[DEBUG] Plotting results to test_outputs/pitch_vs_true.png");
    let root = BitMapBackend::new("test_outputs/pitch_vs_true.png", (800, 600)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let mut chart = ChartBuilder::on(&root)
        .caption("Pitch Detection vs True Pitch", ("sans-serif", 30))
        .margin(40)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(60f32..1100f32, 60f32..1100f32)
        .unwrap();
    chart.configure_mesh()
        .x_desc("True Pitch (Hz)")
        .y_desc("Detected Pitch (Hz)")
        .draw().unwrap();
    chart.draw_series(
        true_pitches.iter().zip(detected_pitches.iter()).map(|(&x, &y)|
            Circle::new((x, y), 6, RED.filled())
        )
    ).unwrap().label("pitch_track").legend(|(x, y)| Circle::new((x, y), 6, RED.filled()));
    chart.draw_series(LineSeries::new(
        (60..1100).map(|v| (v as f32, v as f32)),
        &BLACK,
    )).unwrap().label("Ideal").legend(|(x, y)| PathElement::new(vec![(x-10, y), (x+10, y)], &BLACK));
    chart.configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()
        .unwrap();
    println!("[DEBUG] Plotting complete");
}
