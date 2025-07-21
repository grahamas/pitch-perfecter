use detection::PitchDetector;
use crate::audio::{Audio, MonoAudioSource};

#[derive(Clone, Copy)]
pub struct PitchTracker {
    detector: impl PitchDetector,
    window_size: usize,
    step_size: usize,
}

impl PitchTracker {
    pub fn track_pitch(
        &mut self,
        audio: impl IterableAudio<f64>,
    ) -> Vec<f64> {
        let mut pitches = Vec::new();
        for window in audio.sliding_windows(self.window_size, self.step_size) {
            if let Some(pitch) = self.detector.get_pitch(window) {
                pitches.push(pitch.frequency);
            } else {
                pitches.push(0.0);
            }
        }
        pitches
    }

}

/// Estimate pitch using the track_pitch crate's YIN implementation with custom power and clarity thresholds
pub fn track_pitch(audio: impl IterableAudio, config: PitchTrackerConfig, sample_rate: usize) -> Vec<f64> {
    let PitchTrackerConfig {
        window_size,
        step_size,
        power_threshold,
        clarity_threshold,
    } = config;
    let mut pitches = Vec::new();
    let padding = window_size / 2;
    let mut detector = YINDetector::new(window_size, padding);
    for window in audio.sliding_windows(window_size, step_size) {
        let signal = window.mono_samples();
        if signal.len() < window_size {
            warn!("Not enough samples for pitch detection");
            continue; // Skip if not enough samples
        }
        if let Some(pitch) = detector.get_pitch(&signal, sample_rate, power_threshold, clarity_threshold) {
            pitches.push(pitch.frequency);
        } else {
            pitches.push(0.0);
        }
    }
    pitches
}