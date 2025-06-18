use pitch_detection::detector::yin::YINDetector;
use pitch_detection::detector::PitchDetector;

/// Estimate pitch using the pitch_detection crate's YIN implementation
pub fn pitch_track(signal: &[f32], sample_rate: f32, window_size: usize, step_size: usize, threshold: f32) -> Vec<f32> {
    let mut pitches = Vec::new();
    let mut i = 0;
    let power_threshold = 5.0; // can be tuned
    let clarity_threshold = threshold as f64; // use threshold as clarity threshold
    let padding = window_size / 2;
    let mut detector = YINDetector::new(window_size, padding);
    while i + window_size <= signal.len() {
        let frame: Vec<f64> = signal[i..i+window_size].iter().map(|&x| x as f64).collect();
        if let Some(pitch) = detector.get_pitch(&frame, sample_rate as usize, power_threshold, clarity_threshold) {
            pitches.push(pitch.frequency as f32);
        } else {
            pitches.push(0.0);
        }
        i += step_size;
    }
    pitches
}