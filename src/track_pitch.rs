use pitch_detection::detector::yin::YINDetector;
use pitch_detection::detector::PitchDetector;

#[derive(Clone, Copy)]
pub struct TrackPitchConfig {
    pub window_size: usize,
    pub step_size: usize,
    pub power_threshold: f64,
    pub clarity_threshold: f64,
}

impl TrackPitchConfig {
    pub fn default() -> Self {
        Self {
            window_size: 1024,
            step_size: 256,
            power_threshold: 5.0,
            clarity_threshold: 0.1,
        }
    }
}

/// Estimate pitch using the track_pitch crate's YIN implementation with custom power and clarity thresholds
pub fn track_pitch(signal: &[f32], config: TrackPitchConfig, sample_rate: usize) -> Vec<f64> {
    let TrackPitchConfig {
        window_size,
        step_size,
        power_threshold,
        clarity_threshold,
    } = config;
    let mut pitches = Vec::new();
    let mut i = 0;
    let padding = window_size / 2;
    let mut detector = YINDetector::new(window_size, padding);
    while i + window_size <= signal.len() {
        let frame: Vec<f64> = signal[i..i+window_size].iter().map(|&x| x as f64).collect();
        if let Some(pitch) = detector.get_pitch(&frame, sample_rate, power_threshold, clarity_threshold) {
            pitches.push(pitch.frequency);
        } else {
            pitches.push(0.0);
        }
        i += step_size;
    }
    pitches
}