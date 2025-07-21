use pitch_detection::detector::yin::YINDetector;

pub struct ExternalYinDetector{
    pub power_threshold: f64,
    pub clarity_threshold: f64,
    pub window_size: usize,
    pub padding: usize,
    detector: YINDetector,
}
impl ExternalYinDetector {
    pub fn new(power_threshold: f64, clarity_threshold: f64, window_size: usize, padding: usize) -> Self {
        ExternalYinDetector {
            power_threshold,
            clarity_threshold,
            window_size,
            padding,
            detector: YINDetector::new(window_size, padding),
        }
    }
}
impl MonoPitchDetector for ExternalYinDetector {
    fn get_mono_pitch<T: MonoAudioSource>(&mut self, mono_audio: T) -> Option<Pitch> {
        let sample_rate = mono_audio.sample_rate();
        let signal = mono_audio.mono_samples();
        let pitch = self.detector.get_pitch(&signal, sample_rate, self.power_threshold, self.clarity_threshold);
        pitch
    }
}