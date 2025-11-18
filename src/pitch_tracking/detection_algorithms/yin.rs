use pitch_detection::detector::yin::YINDetector;
use pitch_detection::detector::PitchDetector;
use crate::pitch_tracking::detection::{MonoPitchDetector, Pitch};
use crate::audio::{MonoAudioSource, Audio};


pub struct ExternalYinDetector {
    pub power_threshold: f32,
    pub clarity_threshold: f32,
    pub window_size: usize,
    pub padding: usize,
    detector: YINDetector<f32>,
}
impl ExternalYinDetector {
    pub fn new(power_threshold: f32, clarity_threshold: f32, window_size: usize, padding: usize) -> Self {
        ExternalYinDetector {
            power_threshold,
            clarity_threshold,
            window_size,
            padding,
            detector: YINDetector::<f32>::new(window_size, padding),
        }
    }
}

impl MonoPitchDetector for ExternalYinDetector {
    fn get_mono_pitch<T: MonoAudioSource>(&mut self, mono_audio: T) -> Option<Pitch> {
        let sample_rate = mono_audio.sample_rate();
        let signal = mono_audio.mono_samples();
        
        self.detector.get_pitch(signal, sample_rate as usize, self.power_threshold, self.clarity_threshold)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::MonoAudio;

    #[test]
    fn test_external_yin_detector_sine_wave() {
        // Generate a 440 Hz sine wave
        let sample_rate = 8000;
        let freq = 440.0;
        let n = 1024;
        let signal: Vec<f32> = (0..n)
            .map(|i| (2.0 * std::f32::consts::PI * freq * i as f32 / sample_rate as f32).sin())
            .collect();
        let audio = MonoAudio { samples: signal, sample_rate };
        let mut detector = ExternalYinDetector::new(0.1, 0.9, n, n/2);
        let pitch = detector.get_mono_pitch(audio);
        assert!(pitch.is_some());
        let pitch = pitch.unwrap();
        // Should be close to 440 Hz
        assert!((pitch.frequency - 440.0).abs() < 10.0, "Detected: {}", pitch.frequency);
    }
}