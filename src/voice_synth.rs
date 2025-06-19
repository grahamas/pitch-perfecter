use std::f32::consts::PI;

/// Generate a vibrato sine wave (simulates singing vibrato)
pub fn vibrato_sine_wave(
    base_freq: f32,
    vibrato_freq: f32,
    vibrato_depth: f32,
    sample_rate: f32,
    len: usize,
) -> Vec<f32> {
    (0..len)
        .map(|i| {
            let t = i as f32 / sample_rate;
            let freq = base_freq + vibrato_depth * (2.0 * PI * vibrato_freq * t).sin();
            (2.0 * PI * freq * t).sin()
        })
        .collect()
}

/// Generate a harmonic-rich, voice-like signal with vibrato and amplitude envelope
pub fn voice_like_signal(
    base_freq: f32,
    vibrato_freq: f32,
    vibrato_depth: f32,
    harmonics: usize,
    sample_rate: f32,
    len: usize,
) -> Vec<f32> {
    let mut signal = vec![0.0; len];
    for h in 1..=harmonics {
        let amp = 1.0 / h as f32; // Decreasing amplitude for higher harmonics
        for i in 0..len {
            let t = i as f32 / sample_rate;
            let freq = base_freq * h as f32 + vibrato_depth * (2.0 * PI * vibrato_freq * t).sin();
            signal[i] += amp * (2.0 * PI * freq * t).sin();
        }
    }
    // Apply a simple amplitude envelope (attack/release)
    let attack = (0.05 * len as f32) as usize;
    let release = (0.1 * len as f32) as usize;
    for i in 0..attack {
        signal[i] *= i as f32 / attack as f32;
    }
    for i in (len - release)..len {
        signal[i] *= (len - i) as f32 / release as f32;
    }
    signal
}

/// Generate a voice-like signal with a single pitch (no vibrato, with harmonics and envelope)
pub fn voice_like_single_pitch(
    base_freq: f32,
    harmonics: usize,
    sample_rate: f32,
    len: usize,
) -> Vec<f32> {
    let mut signal = vec![0.0; len];
    for h in 1..=harmonics {
        let amp = 1.0 / h as f32; // Decreasing amplitude for higher harmonics
        for i in 0..len {
            let t = i as f32 / sample_rate;
            let freq = base_freq * h as f32;
            signal[i] += amp * (2.0 * std::f32::consts::PI * freq * t).sin();
        }
    }
    // Apply a simple amplitude envelope (attack/release)
    let attack = (0.05 * len as f32) as usize;
    let release = (0.1 * len as f32) as usize;
    for i in 0..attack {
        signal[i] *= i as f32 / attack as f32;
    }
    for i in (len - release)..len {
        signal[i] *= (len - i) as f32 / release as f32;
    }
    signal
}
