//! # Pitch Tracking Module
//! This module provides functionality for tracking pitch in audio streams using a specified pitch detector.

use crate::audio::{IterableAudio, MonoAudioSource};
use crate::pitch_tracking::detection::MonoPitchDetector;

/// Configuration for pitch tracking
#[derive(Clone, Copy)]
pub struct PitchTracker<D: MonoPitchDetector> {
    detector: D,
    window_size: usize,
    step_size: usize,
}

impl<D: MonoPitchDetector> PitchTracker<D> {
    /// Create a new `PitchTracker` with the specified detector and configuration.
    pub fn new(detector: D, window_size: usize, step_size: usize) -> Self {
        Self {
            detector,
            window_size,
            step_size,
        }
    }
}

/// Track pitches in audio streams using a specified pitch detector and configuration.
impl<D: MonoPitchDetector> PitchTracker<D> {
    pub fn pitches(
        &mut self,
        audio: impl IterableAudio + MonoAudioSource,
    ) -> Vec<f32> {
        let mut pitches = Vec::new();
        for window in audio.sliding_windows(self.window_size, self.step_size) {
            if let Some(pitch) = self.detector.get_mono_pitch(window) {
                pitches.push(pitch.frequency);
            } else {
                pitches.push(0.0);
            }
        }
        pitches
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::MonoAudio;
    use crate::pitch_tracking::detection::{MonoPitchDetector, Pitch};

    // Dummy detector that always returns a fixed pitch
    struct DummyDetector;
    impl MonoPitchDetector for DummyDetector {
        fn get_mono_pitch<T: MonoAudioSource>(&mut self, _audio: T) -> Option<Pitch> {
            Some(Pitch { frequency: 123.0, clarity: 1.0 })
        }
    }

    #[test]
    fn test_pitch_tracker_fixed_pitch() {
        let audio = MonoAudio { samples: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], sample_rate: 44100 };
        let mut tracker = PitchTracker::new(DummyDetector, 2, 2);
        let pitches = tracker.pitches(audio);
        // With window_size=2, step_size=2, expect 3 windows
        assert_eq!(pitches.len(), 3);
        assert!(pitches.iter().all(|&f| (f - 123.0).abs() < 1e-6));
    }

    // Dummy detector that returns None for every window
    struct NoneDetector;
    impl MonoPitchDetector for NoneDetector {
        fn get_mono_pitch<T: MonoAudioSource>(&mut self, _audio: T) -> Option<Pitch> {
            None
        }
    }

    #[test]
    fn test_pitch_tracker_none_pitch() {
        let audio = MonoAudio { samples: vec![1.0, 2.0, 3.0, 4.0], sample_rate: 44100 };
        let mut tracker = PitchTracker::new(NoneDetector, 2, 1);
        let pitches = tracker.pitches(audio);
        // With window_size=2, step_size=1, expect 3 windows
        assert_eq!(pitches.len(), 3);
        assert!(pitches.iter().all(|&f| f == 0.0));
    }

}