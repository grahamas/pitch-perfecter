use pitch_detection::detector::yin::YINDetector;
use pitch_detection::detector::PitchDetector;
use crate::pitch_tracking::detection::{MonoPitchDetector, Pitch};
use audio_utils::MonoAudioSource;
use std::sync::{Arc, Mutex};


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

/// Thread-safe wrapper around YIN detector that can be sent across threads.
/// 
/// This wrapper uses Arc<Mutex<>> to make the detector Send-safe, allowing
/// pitch detection to be performed on the audio callback thread rather than
/// the main thread. The mutex overhead is negligible compared to the YIN
/// algorithm's computation time (~5-10ms).
pub struct ThreadSafeYinDetector {
    power_threshold: f32,
    clarity_threshold: f32,
    window_size: usize,
    padding: usize,
    detector: Arc<Mutex<YINDetector<f32>>>,
}

impl ThreadSafeYinDetector {
    /// Create a new thread-safe YIN detector.
    /// 
    /// # Arguments
    /// * `power_threshold` - Minimum signal power threshold (0.0-1.0)
    /// * `clarity_threshold` - Minimum clarity for pitch detection (0.0-1.0)
    /// * `window_size` - Size of the analysis window in samples
    /// * `padding` - Padding size for the detector
    pub fn new(power_threshold: f32, clarity_threshold: f32, window_size: usize, padding: usize) -> Self {
        ThreadSafeYinDetector {
            power_threshold,
            clarity_threshold,
            window_size,
            padding,
            detector: Arc::new(Mutex::new(YINDetector::<f32>::new(window_size, padding))),
        }
    }
    
    /// Clone the detector reference, allowing it to be shared across threads.
    /// The underlying detector is shared via Arc, so all clones use the same detector.
    pub fn clone_detector(&self) -> Self {
        ThreadSafeYinDetector {
            power_threshold: self.power_threshold,
            clarity_threshold: self.clarity_threshold,
            window_size: self.window_size,
            padding: self.padding,
            detector: Arc::clone(&self.detector),
        }
    }
}

impl MonoPitchDetector for ThreadSafeYinDetector {
    fn get_mono_pitch<T: MonoAudioSource>(&mut self, mono_audio: T) -> Option<Pitch> {
        let sample_rate = mono_audio.sample_rate();
        let signal = mono_audio.mono_samples();
        
        // Lock the detector for the duration of pitch detection
        self.detector.lock().unwrap().get_pitch(
            signal, 
            sample_rate as usize, 
            self.power_threshold, 
            self.clarity_threshold
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use audio_utils::MonoAudio;

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